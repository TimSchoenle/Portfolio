//! The v4 section header: a `grid-12` row with the section number + title in
//! the label column and the intro sentence in the body column.

use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct SectionHeaderProps {
    /// Mono label like "§ 01 — about" (design token, not localized).
    pub num: AttrValue,
    pub title: AttrValue,
    pub intro: AttrValue,
}

#[function_component(SectionHeader)]
pub fn section_header(p: &SectionHeaderProps) -> Html {
    html! {
        <div class="section-header grid-12">
            <div class="col-label">
                <span class="mono text-muted">{ p.num.clone() }</span>
                <h2 class="section-title">{ p.title.clone() }</h2>
            </div>
            <div class="col-body section-intro">{ p.intro.clone() }</div>
        </div>
    }
}
