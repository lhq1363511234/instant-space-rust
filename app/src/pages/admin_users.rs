use leptos::prelude::*;

use crate::components::admin_nav::AdminNav;
use crate::i18n::{t, use_i18n};
use crate::server::admin::{list_admin_users, set_user_role};
use crate::server::auth::current_session;
use instant_domain::admin::AdminUser;
use instant_domain::auth::UserRole;

#[component]
pub fn AdminUsersPage() -> impl IntoView {
    let locale = use_i18n().locale;
    let reload = RwSignal::new(0u32);
    let session = Resource::new(
        || (),
        |_| async move { current_session().await.ok().flatten() },
    );
    let users = Resource::new(
        move || reload.get(),
        |_| async move { list_admin_users().await.unwrap_or_default() },
    );

    view! {
        <main class="page admin-layout">
            <Suspense fallback=move || view! { <p>{move || t(locale.get(), "正在检查管理员权限", "Checking admin access")}</p> }>
                {move || Suspend::new(async move {
                    let user = session.await;
                    if !user.as_ref().is_some_and(|u| u.role.is_admin()) {
                        return view! {
                            <section class="form">
                                <h1>{move || t(locale.get(), "需要管理员登录", "Admin sign-in required")}</h1>
                                <a class="button button-primary" href="/inspace/login">
                                    {move || t(locale.get(), "去登录", "Go to sign in")}
                                </a>
                            </section>
                        }.into_any();
                    }
                    let is_super = user.as_ref().is_some_and(|u| matches!(u.role, UserRole::SuperAdmin));
                    view! {
                        <AdminNav />
                        <section>
                            <h1>{move || t(locale.get(), "用户管理", "User Management")}</h1>
                            <p>{move || if is_super {
                                t(locale.get(), "修改用户角色。只有超级管理员可以调整角色。", "Change user roles. Only super admins can adjust roles.")
                            } else {
                                t(locale.get(), "你可以查看用户，但只有超级管理员能修改角色。", "You can view users, but only super admins can change roles.")
                            }}</p>
                            <Suspense fallback=move || view! { <div class="space-list-skeleton"><span></span><span></span></div> }>
                                {move || Suspend::new(async move {
                                    let items = users.await;
                                    view! { <AdminUserList items=items reload=reload is_super=is_super /> }
                                })}
                            </Suspense>
                        </section>
                    }.into_any()
                })}
            </Suspense>
        </main>
    }
}

#[component]
fn AdminUserList(items: Vec<AdminUser>, reload: RwSignal<u32>, is_super: bool) -> impl IntoView {
    let locale = use_i18n().locale;
    if items.is_empty() {
        return view! {
            <section class="empty-state">
                <strong>{move || t(locale.get(), "没有用户", "No users")}</strong>
            </section>
        }
        .into_any();
    }
    view! {
        <section class="admin-user-list" aria-label="Users">
            <For
                each=move || items.clone()
                key=|u| format!("{}-{:?}", u.id, u.role)
                children=move |u| view! { <AdminUserRow user=u reload=reload is_super=is_super /> }
            />
        </section>
    }
    .into_any()
}

#[component]
fn AdminUserRow(user: AdminUser, reload: RwSignal<u32>, is_super: bool) -> impl IntoView {
    let locale = use_i18n().locale;
    let message = RwSignal::new(None::<String>);
    let error = RwSignal::new(None::<String>);

    let user_id = user.id.to_string();
    let email = user.email.clone();
    let name = user.name.clone().unwrap_or_default();
    let current_role = role_key(&user.role);

    let update = Action::new({
        let user_id = user_id.clone();
        move |role: &String| {
            let user_id = user_id.clone();
            let role = role.clone();
            async move { set_user_role(user_id, role).await }
        }
    });

    Effect::new(move |_| {
        if let Some(result) = update.value().get() {
            match result {
                Ok(()) => {
                    error.set(None);
                    message.set(Some(t(locale.get(), "角色已更新", "Role updated").to_string()));
                    reload.update(|n| *n += 1);
                }
                Err(err) => {
                    message.set(None);
                    error.set(Some(err.to_string()));
                }
            }
        }
    });

    view! {
        <article class="admin-user-row">
            <div class="admin-user-info">
                <strong>{if name.is_empty() { email.clone() } else { name.clone() }}</strong>
                <span class="muted">{email.clone()}</span>
            </div>
            <div class="admin-user-role">
                {if is_super {
                    view! {
                        <select
                            prop:value=current_role
                            on:change=move |ev| { update.dispatch(event_target_value(&ev)); }
                        >
                            <option value="user">{move || t(locale.get(), "普通用户", "User")}</option>
                            <option value="admin">{move || t(locale.get(), "管理员", "Admin")}</option>
                            <option value="super_admin">{move || t(locale.get(), "超级管理员", "Super Admin")}</option>
                        </select>
                    }.into_any()
                } else {
                    view! {
                        <span class=format!("role-badge role-{current_role}")>
                            {role_label(&user.role, locale.get())}
                        </span>
                    }.into_any()
                }}
            </div>
            {move || message.get().map(|m| view! { <p class="form-success">{m}</p> })}
            {move || error.get().map(|e| view! { <p class="form-error">{e}</p> })}
        </article>
    }
}

fn role_key(role: &UserRole) -> &'static str {
    match role {
        UserRole::User => "user",
        UserRole::Admin => "admin",
        UserRole::SuperAdmin => "super_admin",
    }
}

fn role_label(role: &UserRole, locale: crate::i18n::Locale) -> &'static str {
    match role {
        UserRole::User => t(locale, "普通用户", "User"),
        UserRole::Admin => t(locale, "管理员", "Admin"),
        UserRole::SuperAdmin => t(locale, "超级管理员", "Super Admin"),
    }
}
