//! The browser console, the one log the wasm client has.

/// Reports a recoverable problem on the browser console.
///
/// It does nothing outside the wasm client, where the server's own logging covers the same data at
/// start-up.
pub fn warn(message: std::fmt::Arguments<'_>) {
    #[cfg(feature = "web")]
    web_sys::console::warn_1(&message.to_string().into());
    #[cfg(not(feature = "web"))]
    let _ = message;
}
