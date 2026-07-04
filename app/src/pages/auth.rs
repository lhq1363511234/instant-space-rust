use leptos::prelude::*;

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <main class="page">
            <h1>"登录 / 注册"</h1>
            <form class="form">
                <input name="email" type="email" autocomplete="email" aria-label="Email" />
                <input name="password" type="password" autocomplete="current-password" aria-label="Password" />
                <button type="submit">"进入"</button>
            </form>
        </main>
    }
}
