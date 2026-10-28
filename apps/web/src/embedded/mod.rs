//! Build artifacts compiled into the binary: the repository list, the resume fingerprints and the
//! third-party license inventory.
//!
//! `build.rs` copies each artifact into `OUT_DIR`, or writes an empty default when it is absent, so
//! every `include_str!` here resolves on a fresh checkout. Each one is read the same way on the
//! server and the wasm client, which is what keeps the server-rendered page and the hydrated one
//! identical, and what lets the sections that use them render from the first paint without a
//! request.
//!
//! An artifact that fails to parse degrades its section to empty and warns on the browser console;
//! the page still renders.

pub mod licenses;
pub mod repos;
pub mod resume;
