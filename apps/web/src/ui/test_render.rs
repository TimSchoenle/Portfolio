//! Server-side rendering of a single component, for the unit tests of `crate::ui`.
//!
//! A component reads the i18n context and the repository listing that [`crate::app::App`]
//! provides. [`render`] provides the same two around the component under test, renders it once
//! the way the server does, and returns the HTML. Effects never run, so the markup is the first
//! render: the one the server sends and the client hydrates.

use dioxus::prelude::*;
use i18nrs::dioxus::I18nProvider;
use portfolio_data::{LANG_STORAGE_KEY, SITE_LANGUAGES, language_or_default};

/// Renders `component` in `lang` under the providers `App` mounts, and returns its HTML.
pub fn render(lang: &str, component: fn() -> Element) -> String {
    let mut dom = VirtualDom::new_with_props(
        Providers,
        ProvidersProps {
            lang: lang.to_owned(),
            component: Root(component),
        },
    );
    dom.rebuild_in_place();
    dioxus::ssr::render(&dom)
}

/// The translation of the dotted `key` in `lang`, as the component under test reads it.
///
/// Panics when the key is absent, so a test cannot pass by comparing against an empty string.
pub fn text(lang: &str, key: &str) -> String {
    let root: serde_json::Value = serde_json::from_str(language_or_default(lang).translations)
        .expect("the embedded translations parse");
    key.split('.')
        .fold(&root, |value, part| &value[part])
        .as_str()
        .unwrap_or_else(|| panic!("missing translation key '{key}'"))
        .to_owned()
}

/// The component under test, as a prop.
///
/// Never equal to another, so a re-render always re-runs it. Comparing the function pointers
/// themselves would not mean anything: one function can have several addresses.
#[derive(Clone, Copy)]
struct Root(fn() -> Element);

impl PartialEq for Root {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

/// The i18n provider and the repository listing, as `App` mounts them.
#[component]
fn Providers(lang: String, component: Root) -> Element {
    use_context_provider(crate::embedded::repos::load_repos);
    let translations = SITE_LANGUAGES
        .iter()
        .map(|language| (language.code, language.translations))
        .collect();
    rsx! {
        I18nProvider {
            translations,
            default_language: lang,
            storage_name: LANG_STORAGE_KEY.to_string(),
            Under { component }
        }
    }
}

/// Runs `component` in a scope of its own beneath the providers, so its hooks find them.
#[component]
fn Under(component: Root) -> Element {
    (component.0)()
}
