use i18nrs::yew::use_translation;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::router::Route;

#[function_component(NotFound)]
pub fn not_found() -> Html {
    let (i18n, _) = use_translation();

    html! {
        <section class="notfound">
            <div>
                <span class="mono text-muted">{"// signal_lost.404"}</span>
                <h1>{ i18n.t("notFound.title") }</h1>
                <p>{ i18n.t("notFound.description") }</p>
                <Link<Route> to={Route::Home} classes="btn-accent">
                    <span class="mono">{ i18n.t("notFound.home") }</span>
                </Link<Route>>
            </div>
        </section>
    }
}
