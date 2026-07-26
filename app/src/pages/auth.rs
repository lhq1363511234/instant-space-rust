use leptos::prelude::*;

use crate::app_state::refresh_session;
use crate::i18n::{t, use_i18n};
use crate::server::auth::{current_session, login_user, register_user, role_label};

#[component]
pub fn LoginPage() -> impl IntoView {
    let locale = use_i18n().locale;
    let is_register = RwSignal::new(false);
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let name = RwSignal::new(String::new());
    let auth_error = RwSignal::new(None::<String>);
    let current_user = RwSignal::new(None);
    let session = Resource::new(
        || (),
        |_| async move { current_session().await.ok().flatten() },
    );
    let login = Action::new(move |payload: &(String, String)| {
        let (email, password) = payload.clone();
        async move { login_user(email, password).await }
    });
    let register = Action::new(move |payload: &(String, String, Option<String>)| {
        let (email, password, name) = payload.clone();
        async move { register_user(email, password, name).await }
    });

    Effect::new(move |_| {
        if let Some(user) = session.get().flatten() {
            current_user.set(Some(user));
            auth_error.set(None);
        }
    });

    Effect::new(move |_| {
        if let Some(result) = login.value().get() {
            match result {
                Ok(user) => {
                    current_user.set(Some(user));
                    auth_error.set(None);
                    refresh_session();
                }
                Err(err) => auth_error.set(Some(err.to_string())),
            }
        }
    });

    Effect::new(move |_| {
        if let Some(result) = register.value().get() {
            match result {
                Ok(user) => {
                    current_user.set(Some(user));
                    auth_error.set(None);
                    refresh_session();
                }
                Err(err) => auth_error.set(Some(err.to_string())),
            }
        }
    });

    view! {
        <main id="main-content" class="page auth-page">
            <header class="page-head auth-page-head">
                <div>
                    <p class="eyebrow">{move || t(locale.get(), "主理人入口", "Host access")}</p>
                    <h1>{move || t(locale.get(), "登录后创建和管理旅行空间", "Sign in to create and manage travel spaces")}</h1>
                    <p>{move || t(locale.get(), "登录后可以把真实地点创建为空间，维护攻略、密码、二维码分享和社群入口。", "After signing in, create Spaces for real places and manage guides, passwords, QR sharing, and community entry.")}</p>
                </div>
            </header>
            <form
                class="form auth-form"
                on:submit=move |ev| {
                    ev.prevent_default();
                    auth_error.set(None);
                    if is_register.get() {
                        let display_name = name.get().trim().to_string();
                        register.dispatch((
                            email.get(),
                            password.get(),
                            (!display_name.is_empty()).then_some(display_name),
                        ));
                    } else {
                        login.dispatch((email.get(), password.get()));
                    }
                }
            >
                <div class="auth-tabs" role="tablist" aria-label="Authentication mode">
                    <button
                        type="button"
                        class=move || auth_tab_class(!is_register.get())
                        aria-pressed=move || aria_pressed(!is_register.get())
                        on:click=move |_| is_register.set(false)
                    >
                        {move || t(locale.get(), "登录", "Sign in")}
                    </button>
                    <button
                        type="button"
                        class=move || auth_tab_class(is_register.get())
                        aria-pressed=move || aria_pressed(is_register.get())
                        on:click=move |_| is_register.set(true)
                    >
                        {move || t(locale.get(), "注册", "Register")}
                    </button>
                </div>

                {move || is_register.get().then(|| view! {
                    <label class="field-label">
                        <span>{move || t(locale.get(), "显示名称", "Display name")}</span>
                        <input
                            name="name"
                            type="text"
                            autocomplete="name"
                            aria-label="Display name"
                            on:input=move |ev| name.set(event_target_value(&ev))
                        />
                    </label>
                })}

                <label class="field-label">
                    <span>{move || t(locale.get(), "邮箱", "Email")}</span>
                    <input
                        name="email"
                        type="email"
                        autocomplete="email"
                        aria-label="Email"
                        required=true
                        on:input=move |ev| email.set(event_target_value(&ev))
                    />
                </label>
                <label class="field-label">
                    <span>{move || t(locale.get(), "密码", "Password")}</span>
                    <input
                        name="password"
                        type="password"
                        autocomplete=move || if is_register.get() { "new-password" } else { "current-password" }
                        aria-label="Password"
                        required=true
                        minlength="6"
                        on:input=move |ev| password.set(event_target_value(&ev))
                    />
                </label>
                <button class="button button-primary" type="submit">
                    {move || if is_register.get() {
                        t(locale.get(), "创建账户", "Create account")
                    } else {
                        t(locale.get(), "登录", "Sign in")
                    }}
                </button>

                {move || current_user.get().map(|user| view! {
                    <p class="form-success">
                        {format!(
                            "{} {} ({})",
                            t(locale.get(), "已登录：", "Signed in:"),
                            user.email,
                            role_label(&user.role)
                        )}
                    </p>
                })}
                {move || auth_error.get().map(|message| view! { <p class="error">{message}</p> })}
            </form>
        </main>
    }
}

fn auth_tab_class(is_active: bool) -> &'static str {
    if is_active {
        "auth-tab is-active"
    } else {
        "auth-tab"
    }
}

fn aria_pressed(is_active: bool) -> &'static str {
    if is_active {
        "true"
    } else {
        "false"
    }
}
