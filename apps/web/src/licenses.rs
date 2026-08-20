//! The third-party licence inventory, embedded into the binary at build time.
//!
//! `cargo about generate` writes `generated/licenses.json` from `about.toml` and
//! `about.hbs`; `build.rs` copies it into `OUT_DIR` (or writes an empty default
//! when absent), so the include below always resolves. The `/licenses` route
//! renders it — server-side like every other page, from the same artefact that
//! carries the dependencies it names.

use std::sync::LazyLock;

use portfolio_data::licenses::LicensesFile;

/// `licenses.json`, embedded at compile time.
const LICENSES_JSON: &str = include_str!(concat!(env!("OUT_DIR"), "/licenses.json"));

/// The parsed inventory, or `None` when there is nothing to show.
///
/// Parsed once per process rather than per render: this is a quarter of a
/// megabyte of licence text, and on the server `App` mounts once per render, so
/// every incremental-cache miss would otherwise re-parse the whole document.
///
/// Borrowed, not shared through an `Arc` the way `github::REPOS` is: the repo
/// list lives in a Dioxus context that several components clone out of, whereas
/// this has exactly one reader and a `LazyLock` hands it a `&'static` for free.
///
/// The two ways to have nothing — an absent generator run (`build.rs` embedded
/// the empty default) and a document that failed to parse — collapse into one
/// `None` on purpose: the page says the same thing either way, and only the
/// second is worth a console warning.
static LICENSES: LazyLock<Option<LicensesFile>> =
    LazyLock::new(
        || match serde_json::from_str::<LicensesFile>(LICENSES_JSON) {
            Ok(file) if !file.is_empty() => Some(file),
            Ok(_) => None,
            Err(_e) => {
                #[cfg(feature = "web")]
                web_sys::console::warn_1(&format!("licenses.json parse failed: {_e}").into());
                None
            }
        },
    );

/// The embedded inventory, or `None` when this build carries none. Resolves
/// identically on the server (SSR) and the wasm client, so the rendered page
/// matches across hydration.
pub fn load_licenses() -> Option<&'static LicensesFile> {
    LICENSES.as_ref()
}
