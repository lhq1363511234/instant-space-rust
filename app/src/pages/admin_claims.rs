use leptos::prelude::*;

use crate::components::admin_nav::AdminNav;
use crate::i18n::{localize_optional, t, use_i18n};
use crate::server::admin::{approve_host_claim, list_host_claims, reject_host_claim};
use crate::server::auth::current_session;
use instant_domain::admin::HostClaimApplication;

#[component]
pub fn AdminClaimsPage() -> impl IntoView {
    let locale = use_i18n().locale;
    let reload = RwSignal::new(0u32);
    let session = Resource::new(
        || (),
        |_| async move { current_session().await.ok().flatten() },
    );
    let apps = Resource::new(
        move || reload.get(),
        |_| async move { list_host_claims().await.unwrap_or_default() },
    );

    view! {
        <main id="main-content" class="page admin-layout">
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
                    view! {
                        <AdminNav />
                        <section>
                            <h1>{move || t(locale.get(), "主理人认领审批", "Host Claims")}</h1>
                            <p>{move || t(locale.get(), "审批访客提交的主理人认领申请，批准后该用户成为空间主理人，同一空间的其他申请自动作废。", "Approve host claim applications; approval makes the applicant the Space host and voids other pending claims on the same Space.")}</p>
                            <Suspense fallback=move || view! { <div class="space-list-skeleton"><span></span><span></span></div> }>
                                {move || Suspend::new(async move {
                                    let items = apps.await;
                                    view! { <ClaimList items=items reload=reload /> }
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
fn ClaimList(items: Vec<HostClaimApplication>, reload: RwSignal<u32>) -> impl IntoView {
    let locale = use_i18n().locale;
    if items.is_empty() {
        return view! {
            <section class="empty-state">
                <strong>{move || t(locale.get(), "暂无待审批认领申请", "No pending host claims")}</strong>
            </section>
        }
        .into_any();
    }

    view! {
        <section class="my-space-grid" aria-label="Host claims">
            <For
                each=move || items.clone()
                key=|app| app.claim_id
                children=move |app| view! { <ClaimCard app=app reload=reload /> }
            />
        </section>
    }
    .into_any()
}

#[component]
fn ClaimCard(app: HostClaimApplication, reload: RwSignal<u32>) -> impl IntoView {
    let locale = use_i18n().locale;
    let message = RwSignal::new(None::<String>);
    let error = RwSignal::new(None::<String>);

    let name_zh = app.name_zh.clone();
    let name_en = app.name_en.clone();
    let applicant = app
        .applicant_name
        .clone()
        .filter(|v| !v.trim().is_empty())
        .or_else(|| app.applicant_email.clone())
        .unwrap_or_default();
    let note = app.message.clone().filter(|v| !v.trim().is_empty());
    let since = app.created_at.chars().take(10).collect::<String>();
    let claim_id = app.claim_id.to_string();

    let run = move |result: Result<(), ServerFnError>, ok: &str| match result {
        Ok(()) => {
            error.set(None);
            message.set(Some(ok.to_string()));
            reload.update(|n| *n += 1);
        }
        Err(err) => {
            message.set(None);
            error.set(Some(err.to_string()));
        }
    };

    let approve = Action::new({
        let id = claim_id.clone();
        move |_: &()| {
            let id = id.clone();
            async move { approve_host_claim(id).await }
        }
    });
    let reject = Action::new({
        let id = claim_id.clone();
        move |_: &()| {
            let id = id.clone();
            async move { reject_host_claim(id).await }
        }
    });

    Effect::new(move |_| {
        if let Some(result) = approve.value().get() {
            run(result, "已批准");
        }
    });
    Effect::new(move |_| {
        if let Some(result) = reject.value().get() {
            run(result, "已拒绝");
        }
    });

    view! {
        <article class="my-space-card">
            <header>
                <strong>{move || localize_optional(locale.get(), &name_zh, name_en.as_deref())}</strong>
            </header>
            <p class="muted">{move || t(locale.get(), "申请人", "Applicant")}": "{applicant}</p>
            <p class="muted">{move || t(locale.get(), "申请时间", "Applied")}": "{since}</p>
            {note.map(|note| view! {
                <p class="admin-claim-note">"“"{note}"”"</p>
            })}
            <div class="admin-resident-actions">
                <button class="button button-primary" type="button" on:click=move |_| { approve.dispatch(()); }>
                    {move || t(locale.get(), "批准认领", "Approve")}
                </button>
                <button class="button button-danger-light" type="button" on:click=move |_| { reject.dispatch(()); }>
                    {move || t(locale.get(), "拒绝", "Reject")}
                </button>
            </div>
            {move || message.get().map(|m| view! { <p class="form-success">{m}</p> })}
            {move || error.get().map(|e| view! { <p class="form-error">{e}</p> })}
        </article>
    }
}
