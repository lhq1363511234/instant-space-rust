use leptos::prelude::*;

#[component]
pub fn MapHome() -> impl IntoView {
    view! {
        <section class="map-layout">
            <div id="map" class="map-canvas" aria-label="Instant Space map"></div>
            <aside class="space-panel">
                <label>
                    "搜索空间"
                    <input type="search" aria-label="Search spaces" />
                </label>
                <select aria-label="Space type">
                    <option value="">"全部"</option>
                    <option value="scenic">"景点"</option>
                    <option value="food">"美食"</option>
                    <option value="park">"公园"</option>
                    <option value="transit">"交通"</option>
                </select>
            </aside>
        </section>
    }
}
