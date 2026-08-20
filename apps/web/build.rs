//! Embeds `repos.json`, `licenses.json`, `resume-fingerprint.json` and the
//! resume PDFs into the binary.
//!
//! `repos.json` is regenerated from the GitHub API by the `update-repos` crate,
//! `licenses.json` by `cargo about` (see `about.toml`) and
//! `resume-fingerprint.json` + the PDFs by the resume generator. When any of
//! them is absent (dev builds, `cargo clippy`, `cargo test`), an empty default is
//! substituted so the `include_str!`/`include_bytes!`s in `src/github.rs`,
//! `src/licenses.rs` and `src/server/assets.rs` always resolve — on both the wasm
//! client and the native server target.
//!
//! The PDFs are embedded (rather than served from an on-disk `public/resume`
//! directory) so the SSR server stays a single self-contained binary: the
//! `scratch` runtime image ships no writable asset tree, and the non-ASCII
//! resume file names tripped up static file serving.

use std::env;
use std::fs;
use std::path::Path;

use portfolio_data::{OG_IMAGE_FILE, RESUME_FILES};

const EMPTY_MANIFEST: &str = r#"{"algorithm":"","generated_at":"","files":{}}"#;
const EMPTY_REPOS: &str = r#"{"generated_at":"","user":"","repos":[]}"#;
const EMPTY_LICENSES: &str = r#"{"summary":[],"texts":[],"crates":[]}"#;

fn embed(source: &Path, out_dir: &str, name: &str, default: &str) {
    let dest = Path::new(out_dir).join(name);
    let contents = fs::read_to_string(source).unwrap_or_else(|_| default.to_string());
    fs::write(&dest, contents).unwrap_or_else(|_| panic!("write embedded {name}"));
    println!("cargo:rerun-if-changed={}", source.display());
}

/// Copies a binary artifact into `OUT_DIR` under `name` so `assets.rs` can
/// `include_bytes!` it. Writes an empty file when the source is absent (dev
/// builds where the resume generator has not run), which the routes serve as a
/// 404 rather than an empty body.
fn embed_bytes(source: &Path, out_dir: &str, name: &str) {
    let dest = Path::new(out_dir).join(name);
    let bytes = fs::read(source).unwrap_or_default();
    fs::write(&dest, bytes).unwrap_or_else(|_| panic!("write embedded {name}"));
    println!("cargo:rerun-if-changed={}", source.display());
}

/// Copies a resume PDF in under a stable ASCII name (`resume-<lang>.pdf`), so the
/// non-ASCII published file name never has to appear in an `include_bytes!` path.
fn embed_resume(source: &Path, out_dir: &str, lang: &str) {
    embed_bytes(source, out_dir, &format!("resume-{lang}.pdf"));
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

    // The third-party licence inventory, written by `cargo about generate` (see
    // `about.toml` / `about.hbs`, the `licenses` recipe in the justfile and the
    // `generate` stage of the Dockerfile) and rendered by the `/licenses` route.
    //
    // Embedded rather than fetched so the page is part of the server-side render
    // like every other route, and so the attribution a build publishes is the
    // attribution for the dependency set that build actually linked — the two
    // cannot drift apart when they are the same artefact.
    let licenses_source = Path::new(&manifest_dir)
        .join("generated")
        .join("licenses.json");
    embed(&licenses_source, &out_dir, "licenses.json", EMPTY_LICENSES);

    // Resume PDFs, embedded under their canonical `RESUME_FILES` names. The
    // resume generator writes them into `generated/resume/` (see the Dockerfile);
    // absent in dev builds, so an empty file is embedded and served as 404.
    let resume_dir = Path::new(&manifest_dir).join("generated").join("resume");
    for (lang, file_name) in RESUME_FILES {
        embed_resume(&resume_dir.join(file_name), &out_dir, lang);
    }

    // The Open Graph card, from the same generator and embedded for the same
    // reason. Served at `/og-image.png`, which is what the `og:image` meta tag
    // points at.
    let generated = Path::new(&manifest_dir).join("generated");
    embed_bytes(&generated.join(OG_IMAGE_FILE), &out_dir, OG_IMAGE_FILE);
}
