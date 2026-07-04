use leptos::prelude::*;

#[component]
pub fn PrivateVerify(space_name: String) -> impl IntoView {
    view! {
        <form class="form">
            <h2>{space_name}</h2>
            <input name="password" type="password" aria-label="Space password" />
            <button type="submit">"验证"</button>
        </form>
    }
}
