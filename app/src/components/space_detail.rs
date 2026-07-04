use instant_domain::spaces::SpaceSummary;
use leptos::prelude::*;

#[component]
pub fn SpaceDetail(space: SpaceSummary) -> impl IntoView {
    let city = space.city.unwrap_or_default();

    view! {
        <article class="space-detail">
            <h2>{space.name_zh}</h2>
            <p>{city}</p>
        </article>
    }
}
