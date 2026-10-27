//! The third-party license inventory behind the `/licenses` page.
//!
//! `cargo about generate` writes `generated/licenses.json` from `about.toml` and
//! `about.hbs`. The `/licenses` route renders it server-side like every other page,
//! from the same artifact that carries the dependencies it names.

use std::sync::LazyLock;

use portfolio_data::licenses::LicensesFile;

/// `licenses.json`, embedded at compile time.
const LICENSES_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/licenses.json"));

/// The parsed inventory, or `None` when the generator did not run or the document failed to
/// parse.
// Parsed once per process instead of per render: this is a quarter of a megabyte of license text,
// and on the server `App` mounts once per render, so every incremental-cache miss would otherwise
// re-parse the whole document. Borrowed, not shared through an `Arc` the way `super::repos`
// shares the repository list, because this has exactly one reader and a `LazyLock` hands it a
// `&'static` for free.
static LICENSES: LazyLock<Option<LicensesFile>> = LazyLock::new(|| parse(LICENSES_JSON));

/// Parses an inventory, or returns `None` when it lists no third-party crate or does not parse.
// The two ways to have nothing collapse into one `None` because the page says the same thing
// either way; only a parse failure is worth a console warning.
fn parse(json: &str) -> Option<LicensesFile> {
    match serde_json::from_str::<LicensesFile>(json) {
        Ok(file) if !file.is_empty() => Some(file),
        Ok(_) => None,
        Err(err) => {
            crate::console::warn(format_args!("licenses.json parse failed: {err}"));
            None
        }
    }
}

/// Returns the embedded inventory, or `None` when this build carries none.
///
/// It resolves identically on the server and the wasm client, so the rendered page matches across
/// hydration.
pub fn load_licenses() -> Option<&'static LicensesFile> {
    LICENSES.as_ref()
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn the_empty_default_is_none() {
        assert!(parse(r#"{"summary":[],"texts":[],"crates":[]}"#).is_none());
    }

    #[test]
    fn a_malformed_inventory_is_none() {
        assert!(parse("{").is_none());
    }
}
