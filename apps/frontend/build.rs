//! Embeds `resume-fingerprint.json` into the WASM binary.
//!
//! The resume generator writes the real manifest to
//! `apps/frontend/generated/resume-fingerprint.json` before `trunk build`
//! runs. When it is absent (dev builds without resumes, or a bare
//! `cargo clippy`), an empty manifest is substituted so the `include_str!` in
//! `src/github.rs` always resolves.

use std::env;
use std::fs;
use std::path::Path;

const EMPTY_MANIFEST: &str = r#"{"algorithm":"","generated_at":"","files":{}}"#;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set by cargo");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR set by cargo");

    let source = Path::new(&manifest_dir)
        .join("generated")
        .join("resume-fingerprint.json");
    let dest = Path::new(&out_dir).join("resume-fingerprint.json");

    let contents = fs::read_to_string(&source).unwrap_or_else(|_| EMPTY_MANIFEST.to_string());
    fs::write(&dest, contents).expect("write embedded resume-fingerprint.json");

    println!("cargo:rerun-if-changed={}", source.display());
}
