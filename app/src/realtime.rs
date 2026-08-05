//! Phase 6 realtime transport for space rooms.
//!
//! The HTTP server functions remain as a resilient fallback. WebSocket clients
//! use this module for live message delivery and presence updates.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path,
    },
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use futures_util::{SinkExt, StreamExt};
use instant_domain::chat::{ChatMessage, ChatMessageKind};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};
use tokio::sync::{broadcast, Mutex};
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct ChatHub {
    rooms: Arc<Mutex<HashMap<Uuid, Room>>>,
}

struct Room {
    sender: broadcast::Sender<ServerEvent>,
    online: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent {
    History { messages: Vec<ChatMessage> },
    Message { message: ChatMessage },
    Presence { online_count: usize },
    Error { code: String, message: String },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ClientEvent {
    Message {
        body: String,
        #[serde(default)]
        kind: Option<String>,
    },
    Ping,
}

/// Hard ceiling on simultaneous connections in one room. Public spaces can
/// draw a crowd; past this the socket is refused with a clear error instead of
/// degrading every connection in the room (Phase 6 connection cap).
const MAX_ROOM_ONLINE: usize = 300;
/// Minimum interval between messages from one connection (Phase 6 rate limit).
const MIN_SEND_INTERVAL: std::time::Duration = std::time::Duration::from_millis(400);

static CHAT_HUB: OnceLock<ChatHub> = OnceLock::new();

pub fn hub() -> ChatHub {
    CHAT_HUB.get_or_init(ChatHub::default).clone()
}

impl ChatHub {
    async fn join(
        &self,
        space_id: Uuid,
    ) -> (
        broadcast::Sender<ServerEvent>,
        broadcast::Receiver<ServerEvent>,
        usize,
    ) {
        let mut rooms = self.rooms.lock().await;
        let room = rooms.entry(space_id).or_insert_with(|| {
            let (sender, _) = broadcast::channel(256);
            Room { sender, online: 0 }
        });
        room.online += 1;
        let sender = room.sender.clone();
        let receiver = sender.subscribe();
        let online = room.online;
        let _ = sender.send(ServerEvent::Presence {
            online_count: online,
        });
        (sender, receiver, online)
    }

    async fn leave(&self, space_id: Uuid) -> usize {
        let mut rooms = self.rooms.lock().await;
        let Some(room) = rooms.get_mut(&space_id) else {
            return 0;
        };
        room.online = room.online.saturating_sub(1);
        let online = room.online;
        let sender = room.sender.clone();
        let _ = sender.send(ServerEvent::Presence {
            online_count: online,
        });
        if online == 0 {
            rooms.remove(&space_id);
        }
        online
    }

    pub async fn online_count(&self, space_id: Uuid) -> usize {
        let rooms = self.rooms.lock().await;
        rooms.get(&space_id).map(|room| room.online).unwrap_or(0)
    }

    pub async fn publish_message(&self, space_id: Uuid, message: ChatMessage) {
        let rooms = self.rooms.lock().await;
        if let Some(room) = rooms.get(&space_id) {
            let _ = room.sender.send(ServerEvent::Message { message });
        }
    }
}

pub async fn publish_message(space_id: Uuid, message: ChatMessage) {
    hub().publish_message(space_id, message).await;
}

pub async fn space_socket(
    Path(space_id): Path<String>,
    ws: WebSocketUpgrade,
    headers: HeaderMap,
) -> Result<Response, (StatusCode, &'static str)> {
    let space_id =
        Uuid::parse_str(&space_id).map_err(|_| (StatusCode::BAD_REQUEST, "invalid space id"))?;
    if !same_origin_or_non_browser(&headers) {
        return Err((StatusCode::FORBIDDEN, "cross-origin websocket rejected"));
    }
    let pool = instant_space_web_pool().await?;
    let meta = instant_db::spaces::space_access_meta(&pool, space_id)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "database error"))?
        .ok_or((StatusCode::NOT_FOUND, "space not found"))?;

    // Public rooms accept guests. Private rooms require the HttpOnly access
    // cookie established by the existing password verification flow.
    let access_version = if meta.is_public {
        meta.password_version
    } else {
        let name = format!("instant_access_{}", space_id.simple());
        let token = cookie_value(&headers, &name)
            .ok_or((StatusCode::UNAUTHORIZED, "private space access required"))?;
        instant_db::chat::has_valid_access_session(&pool, space_id, &token)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "database error"))?
            .ok_or((StatusCode::UNAUTHORIZED, "private space access expired"))?
    };

    let sender_name = if let Some(token) = cookie_value(&headers, "instant_session") {
        instant_db::users::current_user_by_token(&pool, &token)
            .await
            .ok()
            .flatten()
            .map(|user| user.name.unwrap_or(user.email))
            .unwrap_or_else(|| "Guest".to_string())
    } else {
        "Guest".to_string()
    };

    // Phase 6 connection cap: refuse new sockets when the room is at capacity.
    if hub().online_count(space_id).await >= MAX_ROOM_ONLINE {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "room is full, try again later",
        ));
    }

    Ok(ws
        .on_upgrade(move |socket| client_loop(socket, pool, space_id, sender_name, access_version))
        .into_response())
}

async fn instant_space_web_pool() -> Result<sqlx::PgPool, (StatusCode, &'static str)> {
    crate::server::db_pool()
        .await
        .map_err(|_| (StatusCode::SERVICE_UNAVAILABLE, "database unavailable"))
}

async fn client_loop(
    socket: WebSocket,
    pool: sqlx::PgPool,
    space_id: Uuid,
    sender_name: String,
    access_version: i32,
) {
    let hub = hub();
    let (_room_sender, mut room_receiver, online) = hub.join(space_id).await;
    let _ = instant_db::chat::set_online_count(&pool, space_id, online as i32).await;

    let (mut outgoing, mut incoming) = socket.split();
    let mut last_send = std::time::Instant::now() - MIN_SEND_INTERVAL;

    if let Ok(messages) = instant_db::chat::list_messages(&pool, space_id).await {
        if send_event(&mut outgoing, &ServerEvent::History { messages })
            .await
            .is_err()
        {
            let online = hub.leave(space_id).await;
            let _ = instant_db::chat::set_online_count(&pool, space_id, online as i32).await;
            return;
        }
    }

    loop {
        tokio::select! {
            client_message = incoming.next() => {
                let Some(Ok(client_message)) = client_message else { break; };
                match client_message {
                    Message::Text(text) => {
                        let event = match serde_json::from_str::<ClientEvent>(&text) {
                            Ok(event) => event,
                            Err(_) => {
                                let _ = send_error(&mut outgoing, "invalid_message", "Invalid message format").await;
                                continue;
                            }
                        };
                        match event {
                            ClientEvent::Ping => {
                                if outgoing.send(Message::Pong(Vec::new())).await.is_err() { break; }
                            }
                            ClientEvent::Message { body, kind } => {
                                if last_send.elapsed() < MIN_SEND_INTERVAL {
                                    let _ = send_error(&mut outgoing, "rate_limited", "Please slow down").await;
                                    continue;
                                }
                                last_send = std::time::Instant::now();
                                let body = body.trim().to_string();
                                if body.is_empty() {
                                    let _ = send_error(&mut outgoing, "message_required", "Message is required").await;
                                    continue;
                                }
                                if body.chars().count() > 800 {
                                    let _ = send_error(&mut outgoing, "message_too_long", "Message is too long").await;
                                    continue;
                                }
                                let message_kind = match kind.as_deref() {
                                    Some("help") => ChatMessageKind::Help,
                                    Some("help_resolved") => ChatMessageKind::HelpResolved,
                                    Some("system") => ChatMessageKind::System,
                                    _ => ChatMessageKind::Text,
                                };

                                let Some(meta) = instant_db::spaces::space_access_meta(&pool, space_id).await.ok().flatten() else {
                                    let _ = send_error(&mut outgoing, "space_not_found", "Space no longer exists").await;
                                    break;
                                };
                                // Password rotation does not kick a reader out immediately, but
                                // every send must carry the current version. A stale socket is
                                // closed so the UI can ask for the new password.
                                if !meta.is_public && access_version != meta.password_version {
                                    let _ = send_error(&mut outgoing, "password_changed", "Space password changed; please re-enter password").await;
                                    break;
                                }

                                match instant_db::chat::insert_message(
                                    &pool,
                                    space_id,
                                    sender_name.clone(),
                                    body,
                                    meta.password_version,
                                    message_kind,
                                ).await {
                                    Ok(message) => hub.publish_message(space_id, message).await,
                                    Err(_) => {
                                        let _ = send_error(&mut outgoing, "save_failed", "Could not save message").await;
                                    }
                                }
                            }
                        }
                    }
                    Message::Ping(payload) => {
                        if outgoing.send(Message::Pong(payload)).await.is_err() { break; }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
            room_event = room_receiver.recv() => {
                match room_event {
                    Ok(event) => {
                        if send_event(&mut outgoing, &event).await.is_err() { break; }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }

    let online = hub.leave(space_id).await;
    let _ = instant_db::chat::set_online_count(&pool, space_id, online as i32).await;
}

async fn send_event<S>(outgoing: &mut S, event: &ServerEvent) -> Result<(), ()>
where
    S: futures_util::Sink<Message> + Unpin,
{
    let text = serde_json::to_string(event).map_err(|_| ())?;
    outgoing.send(Message::Text(text)).await.map_err(|_| ())
}

async fn send_error<S>(outgoing: &mut S, code: &str, message: &str) -> Result<(), ()>
where
    S: futures_util::Sink<Message> + Unpin,
{
    send_event(
        outgoing,
        &ServerEvent::Error {
            code: code.to_string(),
            message: message.to_string(),
        },
    )
    .await
}

fn same_origin_or_non_browser(headers: &HeaderMap) -> bool {
    let Some(origin) = headers
        .get(axum::http::header::ORIGIN)
        .and_then(|value| value.to_str().ok())
    else {
        // Non-browser clients do not send Origin. Authentication and access
        // cookies are still validated independently.
        return true;
    };
    let Some(host) = headers
        .get(axum::http::header::HOST)
        .and_then(|value| value.to_str().ok())
    else {
        return false;
    };
    url::Url::parse(origin)
        .ok()
        .and_then(|url| {
            url.host_str()
                .map(str::to_string)
                .map(|name| (name, url.port()))
        })
        .map(|(name, port)| {
            let origin_host = port.map(|port| format!("{name}:{port}")).unwrap_or(name);
            origin_host.eq_ignore_ascii_case(host)
        })
        .unwrap_or(false)
}

fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    let cookie = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    cookie.split(';').find_map(|part| {
        let (cookie_name, value) = part.trim().split_once('=')?;
        (cookie_name == name).then(|| value.to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{
        header::{HOST, ORIGIN},
        HeaderValue,
    };

    #[tokio::test]
    async fn room_presence_increments_and_returns_to_zero() {
        let hub = ChatHub::default();
        let id = Uuid::new_v4();
        let (_, _, first) = hub.join(id).await;
        let (_, _, second) = hub.join(id).await;
        assert_eq!(first, 1);
        assert_eq!(second, 2);
        assert_eq!(hub.leave(id).await, 1);
        assert_eq!(hub.leave(id).await, 0);
    }

    #[test]
    fn browser_websocket_requires_same_origin() {
        let mut same = HeaderMap::new();
        same.insert(HOST, HeaderValue::from_static("opctoai.com"));
        same.insert(ORIGIN, HeaderValue::from_static("https://opctoai.com"));
        assert!(same_origin_or_non_browser(&same));

        let mut cross = HeaderMap::new();
        cross.insert(HOST, HeaderValue::from_static("opctoai.com"));
        cross.insert(ORIGIN, HeaderValue::from_static("https://evil.example"));
        assert!(!same_origin_or_non_browser(&cross));
    }
}
