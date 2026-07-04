use leptos::prelude::*;

#[component]
pub fn SpaceForm() -> impl IntoView {
    view! {
        <form class="form">
            <input name="name_zh" aria-label="Chinese name" />
            <input name="province" aria-label="Province" />
            <input name="city" aria-label="City" />
            <button type="submit">"创建"</button>
        </form>
    }
}
