//! Glue around the i18nrs Dioxus integration.
//!
//! The `I18nProvider` is mounted in [`crate::app::App`]; every component reads
//! the active language and translations through [`use_i18n`], which returns the
//! `I18nContext { i18n, set_language }`. Read a key with `i18n().t("key")` and
//! switch language with `set_language.call("de".into())`.

use dioxus::prelude::*;
use i18nrs::dioxus::I18nContext;

/// Storage key holding the selected language ("en" / "de"). Documented in the
/// privacy policy — keep both in sync.
pub const LANG_STORAGE_KEY: &str = "lang";

/// The i18n context provided by the `I18nProvider`.
pub fn use_i18n() -> I18nContext {
    use_context::<I18nContext>()
}

/// The language to switch to from `current`.
pub fn other_language(current: &str) -> &'static str {
    if current == "de" { "en" } else { "de" }
}
