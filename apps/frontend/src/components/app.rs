use std::collections::HashMap;

use i18nrs::yew::{I18nProvider, use_translation};
use portfolio_data::{I18N_DE, I18N_EN};
use wasm_bindgen::JsCast;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::github::{ReposState, load_repos};
use crate::i18n::{LANG_STORAGE_KEY, ensure_language_seeded, set_document_lang};
use crate::pages::{
    home::Home, imprint::Imprint, not_found::NotFound, privacy_policy::PrivacyPolicy,
};
use crate::router::Route;

use super::footer::Footer;
use super::masthead::Masthead;
use super::palette::CommandPalette;

#[function_component(App)]
pub fn app() -> Html {
    // Must happen before the provider reads storage; see i18n.rs.
    ensure_language_seeded();

    let translations = HashMap::from([("en", I18N_EN), ("de", I18N_DE)]);
    let onchange = Callback::from(|lang: String| set_document_lang(&lang));
    let onerror = Callback::from(|err: String| {
        web_sys::console::warn_1(&format!("i18n: {err}").into());
    });

    html! {
        <BrowserRouter>
            <I18nProvider
                translations={translations}
                default_language={"en".to_string()}
                storage_name={LANG_STORAGE_KEY.to_string()}
                {onchange}
                {onerror}
            >
                <Shell />
            </I18nProvider>
        </BrowserRouter>
    }
}

#[function_component(Shell)]
fn shell() -> Html {
    let (i18n, _) = use_translation();
    let repos = use_state(|| ReposState::Loading);
    let palette_open = use_state(|| false);
    let location = use_location();

    // Fetch repos.json once on mount.
    {
        let repos = repos.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                match load_repos().await {
                    Ok(file) => repos.set(ReposState::Ready(file)),
                    Err(e) => {
                        web_sys::console::warn_1(&format!("repos.json load failed: {e}").into());
                        repos.set(ReposState::Failed);
                    }
                }
            });
        });
    }

    // Mirror the active language onto <html lang> on first render.
    {
        let lang = i18n.get_current_language().to_string();
        use_effect_with((), move |_| set_document_lang(&lang));
    }

    // ⌘K / Ctrl+K toggles the palette, Escape closes it. Re-registered when
    // the open state flips so the closure never reads a stale value.
    {
        let palette_open = palette_open.clone();
        let is_open = *palette_open;
        use_effect_with(is_open, move |_| {
            let cb = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::KeyboardEvent)>::wrap(
                Box::new(move |e: web_sys::KeyboardEvent| {
                    if (e.meta_key() || e.ctrl_key()) && e.key() == "k" {
                        e.prevent_default();
                        palette_open.set(!is_open);
                    } else if e.key() == "Escape" {
                        palette_open.set(false);
                    }
                }),
            );
            let win = web_sys::window().unwrap();
            win.add_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref())
                .ok();
            move || {
                let win = web_sys::window().unwrap();
                win.remove_event_listener_with_callback("keydown", cb.as_ref().unchecked_ref())
                    .ok();
                drop(cb);
            }
        });
    }

    // Scroll to the top whenever the path changes (SPA keeps scroll otherwise).
    {
        let path = location.map(|l| l.path().to_string()).unwrap_or_default();
        use_effect_with(path, |_| {
            if let Some(win) = web_sys::window() {
                win.scroll_to_with_x_and_y(0.0, 0.0);
            }
        });
    }

    let open_palette = {
        let palette_open = palette_open.clone();
        Callback::from(move |_: MouseEvent| palette_open.set(true))
    };
    let close_palette = {
        let palette_open = palette_open.clone();
        Callback::from(move |_: ()| palette_open.set(false))
    };

    let render = {
        let repos = (*repos).clone();
        move |route: Route| match route {
            Route::Home => html! { <Home repos={repos.clone()} /> },
            Route::Imprint => html! { <Imprint /> },
            Route::Privacy => html! { <PrivacyPolicy /> },
            Route::NotFound => html! { <NotFound /> },
        }
    };

    html! {
        <div class="site">
            <Masthead on_open_palette={open_palette} />

            <Switch<Route> render={render} />

            <Footer />

            if *palette_open {
                <CommandPalette repos={repos.repos()} on_close={close_palette} />
            }
        </div>
    }
}
