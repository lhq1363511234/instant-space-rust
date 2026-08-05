use leptos::{leptos_dom::helpers::request_animation_frame, prelude::*};

use crate::app_state::{refresh_session, use_app_refresh_state};
use crate::feedback::use_feedback;
use crate::i18n::{t, use_i18n};
use crate::pages::space::{SpaceExperience, SpacePanel};
use crate::server::auth::{current_session, login_user};

#[derive(Clone, Copy)]
pub struct SpaceExperienceModalState {
    pub open: RwSignal<bool>,
    pub space_id: RwSignal<Option<String>>,
    pub initial_panel: RwSignal<SpacePanel>,
    pub require_login: RwSignal<bool>,
}

impl SpaceExperienceModalState {
    pub fn open_space(self, space_id: impl Into<String>, initial_panel: SpacePanel) {
        self.open_space_with_gate(space_id, initial_panel, false);
    }

    pub fn open_space_with_gate(
        self,
        space_id: impl Into<String>,
        initial_panel: SpacePanel,
        require_login: bool,
    ) {
        self.space_id.set(Some(space_id.into()));
        self.initial_panel.set(initial_panel);
        self.require_login.set(require_login);
        self.open.set(true);
    }

    pub fn close(self) {
        self.open.set(false);
    }
}

pub fn provide_space_experience_modal() -> SpaceExperienceModalState {
    let state = SpaceExperienceModalState {
        open: RwSignal::new(false),
        space_id: RwSignal::new(None),
        initial_panel: RwSignal::new(SpacePanel::Wall),
        require_login: RwSignal::new(false),
    };
    provide_context(state);
    state
}

pub fn use_space_experience_modal() -> Option<SpaceExperienceModalState> {
    use_context::<SpaceExperienceModalState>()
}

#[component]
pub fn SpaceExperienceModalHost(state: SpaceExperienceModalState) -> impl IntoView {
    let locale = use_i18n().locale;
    let refresh = use_app_refresh_state();
    let session = Resource::new(
        move || refresh.session.get(),
        |_| async move { current_session().await.ok().flatten() },
    );
    let session_ready = RwSignal::new(false);
    let signed_in = RwSignal::new(false);
    Effect::new(move |_| {
        if let Some(user) = session.get() {
            signed_in.set(user.is_some());
            session_ready.set(true);
        }
    });

    view! {
        {move || {
            let Some(space_id) = state.open.get().then(|| state.space_id.get()).flatten() else {
                return view! { <span></span> }.into_any();
            };
            let initial_panel = state.initial_panel.get();
            let experience_space_id = space_id.clone();
            let direct_href = format!("/inspace/spaces/{space_id}");
            view! {
                <div
                    class="space-experience-backdrop"
                    role="presentation"
                    tabindex="-1"
                    on:click=move |_| state.close()
                    on:keydown=move |ev| {
                        if ev.key() == "Escape" {
                            state.close();
                        }
                    }
                >
                    <section
                        class="space-experience-dialog"
                        role="dialog"
                        aria-modal="true"
                        aria-labelledby="space-experience-dialog-title"
                        on:click=move |ev| ev.stop_propagation()
                    >
                        <header class="space-experience-dialog-head">
                            <div class="space-experience-dialog-title">
                                <span class="space-experience-dialog-mark" aria-hidden="true"></span>
                                <div>
                                    <p>{move || t(locale.get(), "地点空间", "Place Space")}</p>
                                    <h2 id="space-experience-dialog-title">{move || t(locale.get(), "空间工作区", "Space workspace")}</h2>
                                </div>
                            </div>
                            <div class="space-experience-dialog-actions">
                                <a class="space-experience-direct-link" href=direct_href target="_blank" rel="noreferrer">
                                    {move || t(locale.get(), "新窗口打开", "Open in new tab")}
                                </a>
                                <button
                                    type="button"
                                    class="space-experience-close"
                                    aria-label=move || t(locale.get(), "关闭空间", "Close Space")
                                    autofocus=true
                                    on:click=move |ev| {
                                        ev.stop_propagation();
                                        state.close();
                                    }
                                >
                                    <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 6l12 12M18 6L6 18" /></svg>
                                </button>
                            </div>
                        </header>
                        <div class="space-experience-dialog-scroll">
                            {move || {
                                if state.require_login.get() && !session_ready.get() {
                                    view! {
                                        <div class="space-modal-session-loading" aria-live="polite">
                                            {move || t(locale.get(), "正在确认来客身份……", "Checking your session…")}
                                        </div>
                                    }.into_any()
                                } else if state.require_login.get() && !signed_in.get() {
                                    view! { <SpaceModalLoginGate /> }.into_any()
                                } else {
                                    let experience_space_id = experience_space_id.clone();
                                    view! {
                                        <SpaceExperience
                                            space_id=Signal::derive(move || experience_space_id.clone())
                                            initial_panel=initial_panel
                                            embedded=true
                                        />
                                    }.into_any()
                                }
                            }}
                        </div>
                    </section>
                </div>
            }.into_any()
        }}
    }
}

#[component]
fn SpaceModalLoginGate() -> impl IntoView {
    let locale = use_i18n().locale;
    let feedback = use_feedback();
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let login = Action::new(move |payload: &(String, String)| {
        let (email, password) = payload.clone();
        async move { login_user(email, password).await }
    });

    Effect::new(move |_| {
        if let Some(result) = login.value().get() {
            match result {
                Ok(user) => {
                    error.set(None);
                    let message = format!(
                        "欢迎回来，{}",
                        user.name.as_deref().unwrap_or(user.email.as_str())
                    );
                    request_animation_frame(move || {
                        refresh_session();
                        feedback.success(message);
                    });
                }
                Err(_) => error.set(Some(
                    t(
                        locale.get(),
                        "邮箱或密码不正确，请再试一次。",
                        "The email or password is incorrect. Please try again.",
                    )
                    .to_string(),
                )),
            }
        }
    });

    view! {
        <section class="space-modal-login-gate" aria-labelledby="space-modal-login-title">
            <div class="space-modal-login-copy">
                <p class="space-modal-login-kicker">{move || t(locale.get(), "先认一认来客", "Before entering")}</p>
                <h3 id="space-modal-login-title">{move || t(locale.get(), "登录后，继续进入这个空间", "Sign in, then continue into this Space")}</h3>
                <p>{move || t(locale.get(), "这一步不会带你离开当前页面。登录成功后，空间会在这里自动打开；若空间设有门禁，再继续输入它的访问口令。", "You will stay on this page. After sign-in, the Space opens here automatically; if it has a private gate, enter its access code next.")}</p>
            </div>
            <form
                class="space-modal-login-form"
                on:submit=move |ev| {
                    ev.prevent_default();
                    error.set(None);
                    login.dispatch((email.get(), password.get()));
                }
            >
                <label>
                    <span>{move || t(locale.get(), "邮箱", "Email")}</span>
                    <input
                        name="email"
                        type="email"
                        autocomplete="email"
                        required=true
                        autofocus=true
                        on:input=move |ev| email.set(event_target_value(&ev))
                    />
                </label>
                <label>
                    <span>{move || t(locale.get(), "密码", "Password")}</span>
                    <input
                        name="password"
                        type="password"
                        autocomplete="current-password"
                        required=true
                        minlength="6"
                        on:input=move |ev| password.set(event_target_value(&ev))
                    />
                </label>
                <button class="button button-primary" type="submit" disabled=move || login.pending().get()>
                    {move || if login.pending().get() {
                        t(locale.get(), "正在开门……", "Opening…")
                    } else {
                        t(locale.get(), "登录并继续", "Sign in and continue")
                    }}
                </button>
                {move || error.get().map(|message| view! { <p class="error" role="alert">{message}</p> })}
                <p class="space-modal-login-register">
                    {move || t(locale.get(), "还没有账户？", "New to inspace?")}
                    <a href="/inspace/login">{move || t(locale.get(), "去注册", "Create an account")}</a>
                </p>
            </form>
        </section>
    }
}

#[component]
pub fn OpenSpaceLink(
    space_id: String,
    #[prop(default = SpacePanel::Wall)] initial_panel: SpacePanel,
    #[prop(into)] class: String,
    #[prop(optional, into)] aria_label: Option<String>,
    #[prop(optional)] modal_state: Option<SpaceExperienceModalState>,
    #[prop(default = false)] require_login: bool,
    children: Children,
) -> impl IntoView {
    let modal = modal_state
        .or_else(use_space_experience_modal)
        .unwrap_or_else(provide_space_experience_modal);
    let href = format!("/inspace/spaces/{space_id}");
    let modal_space_id = space_id.clone();

    view! {
        <a
            class=class
            href=href
            aria-label=aria_label
            on:click=move |ev| {
                if ev.button() != 0 || ev.ctrl_key() || ev.meta_key() || ev.shift_key() || ev.alt_key() {
                    return;
                }
                ev.prevent_default();
                modal.open_space_with_gate(modal_space_id.clone(), initial_panel, require_login);
            }
        >
            {children()}
        </a>
    }
}
