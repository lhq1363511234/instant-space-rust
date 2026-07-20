use leptos::prelude::*;

use crate::components::admin_nav::AdminNav;
use crate::i18n::{t, use_i18n};
use crate::server::admin::{
    approve_resident_application, list_resident_applications, reject_resident_application,
};
use crate::server::auth::current_session;
use instant_domain::admin::ResidentApplication;

#[component]
pub fn AdminResidentsPage() -> impl IntoView {
    let locale = use_i18n().locale;
    let reload = RwSignal::new(0u32);
    let session = Resource::new(
        || (),
        |_| async move { current_session().await.ok().flatten() },
    );
    let apps = Resource::new(
        move || reload.get(),
        |_| async move { list_resident_applications().await.unwrap_or_default() },
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
                    view! {
                        <AdminNav />
                        <section>
                            <h1>{move || t(locale.get(), "常驻申请审批", "Resident Applications")}</h1>
                            <p>{move || t(locale.get(), "审批主理人提交的常驻申请，批准后按天数延长空间有效期。", "Approve resident applications; approval extends the space validity by the granted days.")}</p>
                            <Suspense fallback=move || view! { <div class="space-list-skeleton"><span></span><span></span></div> }>
                                {move || Suspend::new(async move {
                                    let items = apps.await;
                                    view! { <ResidentList items=items reload=reload /> }
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
fn ResidentList(items: Vec<ResidentApplication>, reload: RwSignal<u32>) -> impl IntoView {
    let locale = use_i18n().locale;
    if items.is_empty() {
        return view! {
            <section class="empty-state">
                <strong>{move || t(locale.get(), "暂无待审批常驻申请", "No pending resident applications")}</strong>
            </section>
        }
        .into_any();
    }

    view! {
        <section class="my-space-grid" aria-label="Resident applications">
            <For
                each=move || items.clone()
                key=|app| app.space_id
                children=move |app| view! { <ResidentCard app=app reload=reload /> }
            />
        </section>
    }
    .into_any()
}

#[component]
fn ResidentCard(app: ResidentApplication, reload: RwSignal<u32>) -> impl IntoView {
    let locale = use_i18n().locale;
    let message = RwSignal::new(None::<String>);
    let error = RwSignal::new(None::<String>);
    let days = RwSignal::new(app.resident_days.unwrap_or(30).to_string());

    let name = app.name_zh.clone();
    let host = app.host_email.clone().unwrap_or_default();
    let space_id = app.space_id.to_string();

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
        let id = space_id.clone();
        move |_: &()| {
            let id = id.clone();
            let d = days.get().parse::<i32>().unwrap_or(30);
            async move { approve_resident_application(id, d).await }
        }
    });
    let reject = Action::new({
        let id = space_id.clone();
        move |_: &()| {
            let id = id.clone();
            async move { reject_resident_application(id).await }
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
                <strong>{name}</strong>
            </header>
            <p class="muted">{move || t(locale.get(), "申请人", "Applicant")}": "{host}</p>
            <div class="admin-resident-actions">
                <label class="field-label">
                    <span>{move || t(locale.get(), "常驻天数", "Resident days")}</span>
                    <input
                        type="number"
                        min="1"
                        max="3650"
                        prop:value=move || days.get()
                        on:input=move |ev| days.set(event_target_value(&ev))
                    />
                </label>
                <button class="button button-primary" type="button" on:click=move |_| { approve.dispatch(()); }>
                    {move || t(locale.get(), "批准", "Approve")}
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
