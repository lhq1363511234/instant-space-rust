//! Unified user feedback model.
//!
//! Pages and components stop owning ad-hoc `RwSignal<Option<String>>` error
//! toasts and instead push through the app-wide `Feedback` context. The shell
//! renders one toast rail; each message auto-dismisses after a few seconds and
//! can be dismissed by hand. Kinds map to one semantic color each so success
//! and failure never compete with layout-specific error styling.

use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use gloo_timers::future::TimeoutFuture;
#[cfg(feature = "hydrate")]
use leptos::task::spawn_local;

/// Where a feedback message sits on the severity ladder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackKind {
    Success,
    Error,
    Info,
}

/// A single transient message shown in the global toast rail.
#[derive(Debug, Clone)]
pub struct FeedbackMsg {
    pub id: u64,
    pub kind: FeedbackKind,
    pub text: String,
}

#[derive(Debug, Clone, Copy)]
pub struct Feedback {
    pub messages: RwSignal<Vec<FeedbackMsg>>,
}

pub fn provide_feedback() -> Feedback {
    let feedback = Feedback {
        messages: RwSignal::new(Vec::new()),
    };
    provide_context(feedback);
    feedback
}

pub fn use_feedback() -> Feedback {
    use_context::<Feedback>().expect("feedback context should be provided by App")
}

impl Feedback {
    pub fn push(&self, kind: FeedbackKind, text: impl Into<String>) {
        let id = self.next_id();
        self.messages.update(|items| items.push(FeedbackMsg {
            id,
            kind,
            text: text.into(),
        }));
        // Auto-dismiss after 4.5s; enough time to read, short enough to not
        // pile up a second toast behind the first.
        #[allow(unused_variables)]
        let feedback = *self;
        #[cfg(feature = "hydrate")]
        spawn_local(async move {
            TimeoutFuture::new(4500).await;
            feedback.dismiss(id);
        });
    }

    pub fn success(&self, text: impl Into<String>) {
        self.push(FeedbackKind::Success, text);
    }

    pub fn error(&self, text: impl Into<String>) {
        self.push(FeedbackKind::Error, text);
    }

    pub fn info(&self, text: impl Into<String>) {
        self.push(FeedbackKind::Info, text);
    }

    pub fn dismiss(&self, id: u64) {
        self.messages.update(|items| items.retain(|msg| msg.id != id));
    }

    fn next_id(&self) -> u64 {
        // Coarse monotonic id: refresh resets the counter, which is fine for
        // an in-session toast rail.
        let id = self.messages.with_untracked(|items| {
            items
                .last()
                .map(|last| last.id.saturating_add(1))
                .unwrap_or(1)
        });
        id
    }
}

/// One-click dismiss for an individual toast.
#[component]
fn DismissButton(id: u64) -> impl IntoView {
    let feedback = use_feedback();
    view! {
        <button
            type="button"
            class="feedback-dismiss"
            aria-label="关闭提示"
            on:click=move |_| feedback.dismiss(id)
        >"×"</button>
    }
}

/// Global toast rail rendered once in `App`. Visually a fixed bottom-center
/// stack that never competes with page-local inline errors.
#[component]
pub fn FeedbackToasts() -> impl IntoView {
    let feedback = use_feedback();
    view! {
        <div class="feedback-rail" aria-live="polite">
            <For
                each=move || feedback.messages.get()
                key=|msg| msg.id
                children=move |msg: FeedbackMsg| {
                    let kind_class = match msg.kind {
                        FeedbackKind::Success => "is-success",
                        FeedbackKind::Error => "is-error",
                        FeedbackKind::Info => "is-info",
                    };
                    let text = msg.text.clone();
                    view! {
                        <div class=format!("feedback-toast {kind_class}") role=if msg.kind == FeedbackKind::Error { "alert" } else { "status" }>
                            <span class="feedback-text">{text}</span>
                            <DismissButton id=msg.id />
                        </div>
                    }
                }
            />
        </div>
    }
}
