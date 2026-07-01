//! Embeds `repos.json` and `resume-fingerprint.json` into the binary.
//!
//! `repos.json` is regenerated from the GitHub API by the `update-repos` crate
//! and `resume-fingerprint.json` by the resume generator. When either is absent
//! (dev builds, `cargo clippy`, `cargo test`), an empty default is substituted
//! so the `include_str!`s in `src/github.rs` always resolve — on both the wasm
//! client and the native server target.

use std::env;
use std::fs;
use std::path::Path;

const EMPTY_MANIFEST: &str = r#"{"algorithm":"","generated_at":"","files":{}}"#;
const EMPTY_REPOS: &str = r#"{"generated_at":"","user":"","repos":[]}"#;

fn embed(source: &Path, out_dir: &str, name: &str, default: &str) {
    let dest = Path::new(out_dir).join(name);
    let contents = fs::read_to_string(source).unwrap_or_else(|_| default.to_string());
    fs::write(&dest, contents).unwrap_or_else(|_| panic!("write embedded {name}"));
    println!("cargo:rerun-if-changed={}", source.display());
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set by cargo");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR set by cargo");

    let fingerprint_source = Path::new(&manifest_dir)
        .join("generated")
        .join("resume-fingerprint.json");
    embed(
        &fingerprint_source,
        &out_dir,
        "resume-fingerprint.json",
        EMPTY_MANIFEST,
    );

    let repos_source = Path::new(&manifest_dir).join("repos.json");
    embed(&repos_source, &out_dir, "repos.json", EMPTY_REPOS);
}
