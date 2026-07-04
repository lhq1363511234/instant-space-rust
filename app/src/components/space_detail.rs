use instant_domain::spaces::SpaceSummary;
use leptos::prelude::*;

use crate::components::private_verify::PrivateVerify;

#[component]
pub fn SpaceDetail(space: SpaceSummary) -> impl IntoView {
    let city = space.city.unwrap_or_default();
    let is_public = space.is_public;
    let name = space.name_zh.clone();
    let id = space.id.to_string();

    view! {
        <article class="space-detail">
            <h2>{space.name_zh}</h2>
            <p>{city}</p>
            {move || {
                if is_public {
                    view! { <p>"公共空间"</p> }.into_any()
                } else {
                    view! { <PrivateVerify space_id=id.clone() space_name=name.clone() /> }.into_any()
                }
            }}
        </article>
    }
}
