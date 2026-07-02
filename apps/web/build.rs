//! Embeds `repos.json`, `resume-fingerprint.json` and the resume PDFs into the
//! binary.
//!
//! `repos.json` is regenerated from the GitHub API by the `update-repos` crate
//! and `resume-fingerprint.json` + the PDFs by the resume generator. When any of
//! them is absent (dev builds, `cargo clippy`, `cargo test`), an empty default is
//! substituted so the `include_str!`/`include_bytes!`s in `src/github.rs` and
//! `src/server/assets.rs` always resolve — on both the wasm client and the
//! native server target.
//!
//! The PDFs are embedded (rather than served from an on-disk `public/resume`
//! directory) so the SSR server stays a single self-contained binary: the
//! `scratch` runtime image ships no writable asset tree, and the non-ASCII
//! resume file names tripped up static file serving.

use std::env;
use std::fs;
use std::path::Path;

use portfolio_data::RESUME_FILES;

const EMPTY_MANIFEST: &str = r#"{"algorithm":"","generated_at":"","files":{}}"#;
const EMPTY_REPOS: &str = r#"{"generated_at":"","user":"","repos":[]}"#;

fn embed(source: &Path, out_dir: &str, name: &str, default: &str) {
    let dest = Path::new(out_dir).join(name);
    let contents = fs::read_to_string(source).unwrap_or_else(|_| default.to_string());
    fs::write(&dest, contents).unwrap_or_else(|_| panic!("write embedded {name}"));
    println!("cargo:rerun-if-changed={}", source.display());
}

/// Copies a resume PDF into `OUT_DIR` under a stable ASCII name (`resume-<lang>.pdf`)
/// so `assets.rs` can `include_bytes!` it without embedding the non-ASCII source
/// name into the include path. Writes an empty file when the PDF is absent (dev
/// builds where the resume generator has not run), which the route serves as 404.
fn embed_resume(source: &Path, out_dir: &str, lang: &str) {
    let dest = Path::new(out_dir).join(format!("resume-{lang}.pdf"));
    let bytes = fs::read(source).unwrap_or_default();
    fs::write(&dest, bytes).unwrap_or_else(|_| panic!("write embedded resume-{lang}.pdf"));
    println!("cargo:rerun-if-changed={}", source.display());
}

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR set by cargo");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR set by cargo");

    // Ensure the Tailwind output exists so `asset!("/assets/tailwind.css")`
    // resolves even when `npm run build:css` has not been run (e.g. a bare
    // `cargo check`/`clippy`/`test` in CI). The real stylesheet is produced by
    // `npm run build:css` before `dx bundle`; this only writes an empty
    // placeholder when the file is absent and never overwrites a real one.
    let tailwind = Path::new(&manifest_dir).join("assets").join("tailwind.css");
    if !tailwind.exists() {
        fs::write(&tailwind, "/* placeholder — run `npm run build:css` */\n")
            .expect("write tailwind.css placeholder");
    }

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

    // Resume PDFs, embedded under their canonical `RESUME_FILES` names. The
    // resume generator writes them into `generated/resume/` (see the Dockerfile);
    // absent in dev builds, so an empty file is embedded and served as 404.
    let resume_dir = Path::new(&manifest_dir).join("generated").join("resume");
    for (lang, file_name) in RESUME_FILES {
        embed_resume(&resume_dir.join(file_name), &out_dir, lang);
    }
}
