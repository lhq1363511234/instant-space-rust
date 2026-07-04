use leptos::prelude::*;

#[component]
pub fn GuideBrowser() -> impl IntoView {
    view! {
        <section class="guide-browser">
            <h1>"导览"</h1>
            <div class="filter-row">
                <select aria-label="Province"><option value="">"省份"</option></select>
                <select aria-label="City"><option value="">"城市"</option></select>
                <select aria-label="District"><option value="">"区域"</option></select>
                <select aria-label="Spot"><option value="">"地点"</option></select>
            </div>
        </section>
    }
}
