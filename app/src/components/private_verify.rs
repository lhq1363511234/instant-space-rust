use leptos::prelude::*;

use crate::server::chat::verify_space_password;

#[component]
pub fn PrivateVerify(space_id: String, space_name: String) -> impl IntoView {
    let password = RwSignal::new(String::new());
    let verified_version = RwSignal::new(None::<i32>);
    let error = RwSignal::new(None::<String>);
    let verify = Action::new(move |submitted: &String| {
        let space_id = space_id.clone();
        let submitted = submitted.clone();
        async move { verify_space_password(space_id, submitted).await }
    });

    Effect::new(move |_| {
        if let Some(result) = verify.value().get() {
            match result {
                Ok(version) => {
                    verified_version.set(Some(version));
                    error.set(None);
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
            aria-label="Private space verification"
            on:submit=move |ev| {
                ev.prevent_default();
                verify.dispatch(password.get());
            }
        >
            <div class="private-entry-header">
                <span class="space-badge space-badge-private">"Private"</span>
                <h3>{space_name}</h3>
            </div>
            <label class="field-label">
                <span>"Access code"</span>
                <input
                    name="password"
                    type="password"
                    aria-label="Private space password"
                    placeholder="Enter space password"
                    on:input=move |ev| password.set(event_target_value(&ev))
                />
            </label>
            <button class="button button-primary" type="submit">"Enter chat"</button>
            {move || {
                verified_version
                    .get()
                    .map(|_| view! { <p class="form-success">"Access unlocked."</p> })
            }}
            {move || error.get().map(|message| view! { <p class="error">{message}</p> })}
        </form>
    }
}
