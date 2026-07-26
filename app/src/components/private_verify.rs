use leptos::prelude::*;

use crate::i18n::{t, use_i18n};
use crate::server::chat::verify_space_password;

#[component]
pub fn PrivateVerify(
    space_id: String,
    space_name: String,
    #[prop(optional)] on_verified: Option<Callback<()>>,
) -> impl IntoView {
    let locale = use_i18n().locale;
    let password = RwSignal::new(String::new());
    let verified_version = RwSignal::new(None::<i32>);
    let error = RwSignal::new(None::<String>);
    let verify_space_id = space_id.clone();
    let chat_href = format!("/inspace/spaces/{space_id}");
    let verify = Action::new(move |submitted: &String| {
        let space_id = verify_space_id.clone();
        let submitted = submitted.clone();
        async move { verify_space_password(space_id, submitted).await }
    });

    Effect::new(move |_| {
        if let Some(result) = verify.value().get() {
            match result {
                Ok(grant) => {
                    verified_version.set(Some(grant.password_version));
                    error.set(None);
                    if let Some(callback) = on_verified {
                        callback.run(());
                    }
                }
                Err(err) => {
                    verified_version.set(None);
                    error.set(Some(err.to_string()));
                }
            }
        }
    });

    view! {
        <form
            class="private-entry"
            aria-label=move || t(locale.get(), "私密空间验证", "Private space verification")
            on:submit=move |ev| {
                ev.prevent_default();
                verify.dispatch(password.get());
            }
        >
            <div class="private-entry-header">
                <span class="space-badge space-badge-private">{move || t(locale.get(), "私密", "Private")}</span>
                <h3>{space_name}</h3>
            </div>
            <label class="field-label">
                <span>{move || t(locale.get(), "访问码", "Access code")}</span>
                <input
                    name="password"
                    type="password"
                    aria-label=move || t(locale.get(), "私密空间密码", "Private space password")
                    placeholder=move || t(locale.get(), "输入空间密码", "Enter space password")
                    on:input=move |ev| password.set(event_target_value(&ev))
                />
            </label>
            <button class="button button-primary" type="submit">{move || t(locale.get(), "进入聊天", "Enter chat")}</button>
            {move || {
                verified_version
                    .get()
                    .map(|_| view! {
                        <div class="form-success">
                            <p>{move || t(locale.get(), "访问已解锁。", "Access unlocked.")}</p>
                            <a class="button button-primary" href=chat_href.clone()>
                                {move || t(locale.get(), "进入私密聊天", "Open private chat")}
                            </a>
                        </div>
                    })
            }}
            {move || error.get().map(|message| view! { <p class="error">{message}</p> })}
        </form>
    }
}
