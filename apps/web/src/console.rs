//! The browser console, the one log the wasm client has.

/// Reports a recoverable problem on the browser console. A no-op outside the wasm client,
/// where the server's own logging covers the same data at start-up.
pub fn warn(message: std::fmt::Arguments<'_>) {
    // Gated on the target as well as the feature: `--all-features` enables `web` in a native
    // build, where every `web_sys` call aborts the process.
    #[cfg(all(feature = "web", target_arch = "wasm32"))]
    web_sys::console::warn_1(&message.to_string().into());
    #[cfg(not(all(feature = "web", target_arch = "wasm32")))]
    let _ = message;
}
