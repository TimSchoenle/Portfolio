use i18nrs::yew::use_translation;
use portfolio_data::CONFIG;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::router::Route;

#[function_component(Footer)]
pub fn footer() -> Html {
    let (i18n, _) = use_translation();
    let url_display = CONFIG
        .url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let year = js_sys::Date::new_0().get_full_year();

    html! {
        <footer class="site-footer">
            <div class="footer-top">
                <div>
                    <span class="mono text-fg">{ format!("{} · PORTFOLIO", CONFIG.full_name.to_uppercase()) }</span>
                    <div class="footer-tagline">{ i18n.t("footer.credit") }</div>
                </div>
                <div class="footer-cols">
                    <div class="footer-col">
                        <span class="mono text-muted">{"SOCIAL"}</span>
                        <a href={CONFIG.github} target="_blank" rel="noreferrer">{"GitHub"}</a>
                        <a href={CONFIG.linkedin} target="_blank" rel="noreferrer">{"LinkedIn"}</a>
                        <a href={CONFIG.url} target="_blank" rel="noreferrer">{url_display}</a>
                    </div>
                    <div class="footer-col">
                        <span class="mono text-muted">{"LEGAL"}</span>
                        <Link<Route> to={Route::Imprint}>{ i18n.t("footer.imprint") }</Link<Route>>
                        <Link<Route> to={Route::Privacy}>{ i18n.t("footer.privacy") }</Link<Route>>
                        <a href={CONFIG.repository} target="_blank" rel="noreferrer">{ i18n.t("footer.colophon") }</a>
                    </div>
                    <div class="footer-col">
                        <span class="mono text-muted">{"META"}</span>
                        <span>{ format!("© {year}") }</span>
                        <span>{ i18n.t("common.country") }</span>
                        <span>{ i18n.t("footer.built") }</span>
                    </div>
                </div>
            </div>
            <div class="footer-bottom">
                <span class="mono text-muted">{"— END OF TRANSMISSION —"}</span>
            </div>
        </footer>
    }
}
