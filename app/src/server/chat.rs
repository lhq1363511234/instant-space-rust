use instant_domain::chat::ChatMessage;
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use instant_auth::verify_password;
#[cfg(feature = "ssr")]
use uuid::Uuid;

#[server(VerifySpacePassword, "/api")]
pub async fn verify_space_password(
    space_id: String,
    password: String,
) -> Result<i32, ServerFnError> {
    let id = Uuid::parse_str(&space_id).map_err(|err| ServerFnError::new(err.to_string()))?;
    let database_url =
        std::env::var("DATABASE_URL").map_err(|err| ServerFnError::new(err.to_string()))?;
    let pool = instant_db::connect(&database_url)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;
    let Some((hash, version)) = instant_db::spaces::space_password_hash(&pool, id)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?
    else {
        return Err(ServerFnError::new("space not found"));
    };
    let ok =
        verify_password(&password, &hash).map_err(|err| ServerFnError::new(err.to_string()))?;
    if ok {
        Ok(version)
    } else {
        Err(ServerFnError::new("wrong password"))
    }
}

#[server(ListChatMessages, "/api")]
pub async fn list_chat_messages(space_id: String) -> Result<Vec<ChatMessage>, ServerFnError> {
    let id = Uuid::parse_str(&space_id).map_err(|err| ServerFnError::new(err.to_string()))?;
    let database_url =
        std::env::var("DATABASE_URL").map_err(|err| ServerFnError::new(err.to_string()))?;
    let pool = instant_db::connect(&database_url)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;

    instant_db::chat::list_messages(&pool, id)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))
}
