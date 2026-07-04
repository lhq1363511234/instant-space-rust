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
            class="form"
            on:submit=move |ev| {
                ev.prevent_default();
                verify.dispatch(password.get());
            }
        >
            <h2>{space_name}</h2>
            <input
                name="password"
                type="password"
                aria-label="Space password"
                on:input=move |ev| password.set(event_target_value(&ev))
            />
            <button type="submit">"验证"</button>
            {move || {
                verified_version
                    .get()
                    .map(|version| view! { <p>"已验证，版本 "{version}</p> })
            }}
            {move || error.get().map(|message| view! { <p class="error">{message}</p> })}
        </form>
    }
}
