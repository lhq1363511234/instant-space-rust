use leptos::prelude::*;

use crate::components::admin_nav::AdminNav;
use crate::i18n::{t, use_i18n};
use crate::server::admin::{get_admin_stats, list_audit_log};
use crate::server::auth::current_session;

#[component]
pub fn AdminRoutes() -> impl IntoView {
    let locale = use_i18n().locale;
    let session = Resource::new(
        || (),
        |_| async move { current_session().await.ok().flatten() },
    );
    let stats = Resource::new(|| (), |_| async move { get_admin_stats().await.ok() });
    let audit = Resource::new(|| (), |_| async move { list_audit_log().await.unwrap_or_default() });

    view! {
        <main class="page admin-layout">
            <Suspense fallback=move || view! { <p>{move || t(locale.get(), "正在检查管理员权限", "Checking admin access")}</p> }>
                {move || Suspend::new(async move {
                    let user = session.await;
                    if user.as_ref().is_some_and(|user| user.role.is_admin()) {
                        view! {
                            <AdminNav />
                            <section>
                                <h1>{move || t(locale.get(), "管理后台", "Admin Dashboard")}</h1>
                                <p>{move || t(locale.get(), "仪表盘、空间、攻略、模板和常驻申请共用这个后台外壳。", "Dashboard, spaces, guides, templates, and resident applications share this shell.")}</p>
                                <Suspense fallback=move || view! { <p>{move || t(locale.get(), "正在加载统计", "Loading stats")}</p> }>
                                    {move || Suspend::new(async move {
                                        let stats = stats.await;
                                        view! {
                                            <div class="stats-grid">
                                                <article>
                                                    <strong>{move || t(locale.get(), "空间", "Spaces")}</strong>
                                                    <span>{stats.as_ref().map(|s| s.spaces_count).unwrap_or_default()}</span>
                                                </article>
                                                <article>
                                                    <strong>{move || t(locale.get(), "攻略", "Guides")}</strong>
                                                    <span>{stats.as_ref().map(|s| s.guides_count).unwrap_or_default()}</span>
                                                </article>
                                                <article>
                                                    <strong>{move || t(locale.get(), "用户", "Users")}</strong>
                                                    <span>{stats.as_ref().map(|s| s.users_count).unwrap_or_default()}</span>
                                                </article>
                                                <article>
                                                    <strong>{move || t(locale.get(), "常驻", "Resident")}</strong>
                                                    <span>{stats.as_ref().map(|s| s.pending_resident_applications).unwrap_or_default()}</span>
                                                </article>
                                            </div>
                                        }
                                    })}
                                </Suspense>
                                <h2>{move || t(locale.get(), "操作日志", "Audit Log")}</h2>
                                <p>{move || t(locale.get(), "记录改角色、驻留审批等敏感操作。", "Records privileged actions such as role changes and resident approvals.")}</p>
                                <Suspense fallback=move || view! { <p>{move || t(locale.get(), "正在加载日志", "Loading log")}</p> }>
                                    {move || Suspend::new(async move {
                                        let entries = audit.await;
                                        if entries.is_empty() {
                                            view! { <p class="muted">{move || t(locale.get(), "暂无操作记录。", "No audit entries yet.")}</p> }.into_any()
                                        } else {
                                            view! {
                                                <table class="admin-audit-table">
                                                    <thead>
                                                        <tr>
                                                            <th>{move || t(locale.get(), "时间", "Time")}</th>
                                                            <th>{move || t(locale.get(), "操作者", "Actor")}</th>
                                                            <th>{move || t(locale.get(), "动作", "Action")}</th>
                                                            <th>{move || t(locale.get(), "对象", "Target")}</th>
                                                            <th>{move || t(locale.get(), "详情", "Detail")}</th>
                                                        </tr>
                                                    </thead>
                                                    <tbody>
                                                        <For
                                                            each=move || entries.clone()
                                                            key=|e| e.id
                                                            children=move |e| view! {
                                                                <tr>
                                                                    <td>{e.created_at}</td>
                                                                    <td>{e.actor_email.unwrap_or_else(|| "—".to_string())}</td>
                                                                    <td>{e.action}</td>
                                                                    <td>{format!("{}/{}", e.target_type, e.target_id.unwrap_or_default())}</td>
                                                                    <td>{e.detail.unwrap_or_default()}</td>
                                                                </tr>
                                                            }
                                                        />
                                                    </tbody>
                                                </table>
                                            }.into_any()
                                        }
                                    })}
                                </Suspense>
                            </section>
                        }.into_any()
                    } else {
                        view! {
                            <section class="form">
                                <h1>{move || t(locale.get(), "需要管理员登录", "Admin sign-in required")}</h1>
                                <p>{move || t(locale.get(), "请使用最高权限管理员账户登录后再访问后台。", "Sign in with a privileged admin account before opening the dashboard.")}</p>
                                <a class="button button-primary" href="/inspace/login">
                                    {move || t(locale.get(), "去登录", "Go to sign in")}
                                </a>
                            </section>
                        }.into_any()
                    }
                })}
            </Suspense>
        </main>
    }
}
