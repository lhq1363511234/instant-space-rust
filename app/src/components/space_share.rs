use leptos::prelude::*;

use crate::i18n::{t, use_i18n};

/// Stable public share path for a space landing page.
pub fn space_share_path(space_id: &str) -> String {
    format!("/inspace/spaces/{space_id}")
}

pub fn space_share_url(space_id: &str) -> String {
    let path = space_share_path(space_id);
    let origin = instant_map_ui::page_origin();
    if origin.trim().is_empty() {
        path
    } else {
        format!("{}{}", origin.trim_end_matches('/'), path)
    }
}

fn qr_image_url(share_url: &str) -> String {
    format!(
        "https://api.qrserver.com/v1/create-qr-code/?size=180x180&margin=8&data={}",
        urlencoding_minimal(share_url)
    )
}

fn urlencoding_minimal(value: &str) -> String {
    let mut out = String::with_capacity(value.len() * 3);
    for b in value.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

#[component]
pub fn SpaceSharePanel(
    space_id: String,
    #[prop(optional)] space_name: Option<String>,
    #[prop(default = false)] compact: bool,
) -> impl IntoView {
    let locale = use_i18n().locale;
    let copied = RwSignal::new(false);
    let show_qr = RwSignal::new(!compact);
    let share_url = RwSignal::new(space_share_url(&space_id));
    let path_only = space_share_path(&space_id);
    let _space_name = space_name.unwrap_or_default();

    // Refresh absolute URL after hydrate (origin available in browser).
    Effect::new(move |_| {
        let next = space_share_url(&space_id);
        if !next.is_empty() {
            share_url.set(next);
        }
    });

    let class_name = if compact {
        "space-share-panel is-compact"
    } else {
        "space-share-panel"
    };

    view! {
        <section class=class_name aria-label=move || t(locale.get(), "分享空间", "Share space")>
            <div class="space-share-head">
                <div>
                    <strong>{move || t(locale.get(), "分享链接 / 二维码", "Share link / QR")}</strong>
                    <p>
                        {move || t(
                            locale.get(),
                            "把链接或二维码发给朋友，扫码后进入这个空间详情页（不是单篇攻略）。",
                            "Send the link or QR to friends. Scanning opens this space page, not a single guide.",
                        )}
                    </p>
                </div>
            </div>

            <label class="field-label space-share-url-field">
                <span>{move || t(locale.get(), "空间链接", "Space link")}</span>
                <div class="space-share-url-row">
                    <input
                        type="text"
                        readonly
                        prop:value=move || share_url.get()
                        aria-label=move || t(locale.get(), "空间分享链接", "Space share link")
                    />
                    <button
                        type="button"
                        class="button button-primary"
                        on:click=move |_| {
                            let url = share_url.get();
                            if instant_map_ui::copy_text(&url) {
                                copied.set(true);
                            }
                        }
                    >
                        {move || if copied.get() {
                            t(locale.get(), "已复制", "Copied")
                        } else {
                            t(locale.get(), "复制链接", "Copy link")
                        }}
                    </button>
                </div>
                <small class="space-share-path-hint">{path_only}</small>
            </label>

            <div class="space-share-qr-actions">
                <button
                    type="button"
                    class="button button-secondary-light"
                    on:click=move |_| show_qr.update(|v| *v = !*v)
                >
                    {move || if show_qr.get() {
                        t(locale.get(), "收起二维码", "Hide QR")
                    } else {
                        t(locale.get(), "显示二维码", "Show QR")
                    }}
                </button>
                <a
                    class="button button-secondary-light"
                    prop:href=move || share_url.get()
                    target="_blank"
                    rel="noreferrer"
                >
                    {move || t(locale.get(), "打开分享页", "Open share page")}
                </a>
            </div>

            {move || if show_qr.get() {
                let qr_src = qr_image_url(&share_url.get());
                view! {
                    <div class="space-share-qr">
                        <img
                            src=qr_src
                            width="180"
                            height="180"
                            alt=move || t(locale.get(), "空间分享二维码", "Space share QR code")
                        />
                        <p>
                            {move || t(
                                locale.get(),
                                "手机扫码即可进入空间；可用于店内物料或发给朋友。",
                                "Scan with a phone to open the space. Use for store prints or friend shares.",
                            )}
                        </p>
                    </div>
                }.into_any()
            } else {
                view! { <span class="space-share-qr-placeholder" aria-hidden="true"></span> }.into_any()
            }}
        </section>
    }
}
