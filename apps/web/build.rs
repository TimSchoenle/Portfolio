//! Embeds the build-time generated data into the binary: `repos.json`, `licenses.json`,
//! `resume-fingerprint.json`, the resume PDFs, the Open Graph card and the app icons.
//!
//! `repos.json` comes from the `update-repos` crate, `licenses.json` from `cargo about` (see
//! `about.toml`) and everything else from the resume generator. Each is copied into `OUT_DIR`,
//! where `src/github.rs`, `src/licenses.rs` and `src/server/assets.rs` include it; the resume
//! PDFs and the icons are additionally listed in generated Rust tables (`resumes.rs`,
//! `app_icons.rs`) so the server serves exactly what the data crate publishes.
//!
//! # Two modes
//!
//! A development build (`cargo check`, `clippy`, `test`, `dx serve`) has usually not run the
//! generators, so a missing input is replaced by an empty default and the routes serving it
//! answer 404.
//!
//! An image build must not do that silently: an empty `licenses.json` publishes a licenses page
//! that attributes nothing, and an empty stylesheet ships an unstyled site. With
//! `PORTFOLIO_REQUIRE_GENERATED=1` — which the `Dockerfile` sets — every missing input is an
//! error naming the step that should have produced it.

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use portfolio_data::{APP_ICONS, OG_IMAGE_FILE, SITE_LANGUAGES};

/// Set to `1` to turn a missing generated input into a build error.
const REQUIRE_ENV: &str = "PORTFOLIO_REQUIRE_GENERATED";

const EMPTY_MANIFEST: &str = r#"{"algorithm":"","generated_at":"","files":{}}"#;
const EMPTY_REPOS: &str = r#"{"generated_at":"","user":"","repos":[]}"#;
const EMPTY_LICENSES: &str = r#"{"summary":[],"texts":[],"crates":[]}"#;

/// Where the inputs come from and where they go, and whether a missing one is fatal.
struct Embedder {
    crate_dir: PathBuf,
    out_dir: PathBuf,
    strict: bool,
    /// Inputs that were absent, with the step that produces each.
    missing: Vec<String>,
}

impl Embedder {
    /// Copies `source` (relative to the crate) into `OUT_DIR` as `name`, or writes `default`
    /// when it is absent. `producer` names what creates it, for the strict-mode error.
    fn embed(&mut self, source: &str, name: &str, default: &[u8], producer: &str) -> PathBuf {
        let source = self.crate_dir.join(source);
        let dest = self.out_dir.join(name);
        let bytes = if let Ok(bytes) = fs::read(&source) {
            bytes
        } else {
            self.missing
                .push(format!("{} (produced by {producer})", source.display()));
            default.to_vec()
        };
        fs::write(&dest, bytes).unwrap_or_else(|err| panic!("write embedded {name}: {err}"));
        self.watch(&source);
        dest
    }

    /// Re-runs this script when `path` changes — or, when it does not exist yet, when the
    /// nearest existing directory above it inside the crate does. Watching a missing path
    /// directly makes Cargo re-run the script on every build.
    fn watch(&self, path: &Path) {
        let mut target = path;
        while !target.exists() {
            match target.parent() {
                Some(parent) if parent.starts_with(&self.crate_dir) && parent != self.crate_dir => {
                    target = parent;
                }
                _ => return,
            }
        }
        println!("cargo:rerun-if-changed={}", target.display());
    }

    /// Fails the build in strict mode when anything was missing.
    fn finish(self) {
        assert!(
            !self.strict || self.missing.is_empty(),
            "{REQUIRE_ENV}=1 but these generated inputs are missing:\n  {}",
            self.missing.join("\n  ")
        );
    }
}

fn main() {
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("set by cargo"));
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("set by cargo"));
    println!("cargo:rerun-if-env-changed={REQUIRE_ENV}");
    let strict = env::var(REQUIRE_ENV).is_ok_and(|value| value == "1");

    let mut embedder = Embedder {
        crate_dir,
        out_dir,
        strict,
        missing: Vec::new(),
    };

    // `asset!("/assets/tailwind.css")` needs the file at compile time. `npm run build:css`
    // writes it before `dx bundle`; a bare `cargo check` has not run npm, so a development build
    // gets an empty placeholder in its place. The one write outside `OUT_DIR`, and only ever of
    // a file that is gitignored and absent.
    let tailwind = embedder.crate_dir.join("assets").join("tailwind.css");
    if !tailwind.exists() {
        embedder.missing.push(format!(
            "{} (produced by `npm run build:css`)",
            tailwind.display()
        ));
        if !strict {
            fs::write(&tailwind, "/* placeholder — run `npm run build:css` */\n")
                .expect("write tailwind.css placeholder");
        }
    }

    embedder.embed(
        "generated/resume-fingerprint.json",
        "resume-fingerprint.json",
        EMPTY_MANIFEST.as_bytes(),
        "resume-generator",
    );
    embedder.embed(
        "repos.json",
        "repos.json",
        EMPTY_REPOS.as_bytes(),
        "update-repos",
    );
    // The attribution a build publishes has to be the attribution for the dependency set that
    // build linked, so it is embedded rather than fetched.
    embedder.embed(
        "generated/licenses.json",
        "licenses.json",
        EMPTY_LICENSES.as_bytes(),
        "`cargo about generate` (just licenses)",
    );
    embedder.embed(
        &format!("generated/{OG_IMAGE_FILE}"),
        OG_IMAGE_FILE,
        b"",
        "resume-generator",
    );

    // One resume per site language, under a stable ASCII name so the non-ASCII published name
    // never appears in an `include_bytes!` path.
    let mut resumes = String::from(
        "/// Every site language's resume: `(language, published file name, PDF bytes)`.\n\
         /// Generated by `build.rs` from `portfolio_data::SITE_LANGUAGES`.\n\
         const RESUMES: [(&str, &str, &[u8]); SITE_LANGUAGE_COUNT] = [\n",
    );
    for language in SITE_LANGUAGES {
        let dest = embedder.embed(
            &format!("generated/resume/{}", language.resume_file),
            &format!("resume-{}.pdf", language.code),
            b"",
            "resume-generator",
        );
        let _ = writeln!(
            resumes,
            "    ({:?}, {:?}, include_bytes!({:?})),",
            language.code,
            language.resume_file,
            dest.display().to_string()
        );
    }
    resumes.push_str("];\n");
    resumes = resumes.replace("SITE_LANGUAGE_COUNT", &SITE_LANGUAGES.len().to_string());
    fs::write(embedder.out_dir.join("resumes.rs"), resumes).expect("write resumes.rs");

    let mut icons = String::from(
        "/// The web manifest's raster icons: `(file name, PNG bytes)`.\n\
         /// Generated by `build.rs` from `portfolio_data::APP_ICONS`.\n",
    );
    let _ = writeln!(
        icons,
        "const APP_ICON_FILES: [(&str, &[u8]); {}] = [",
        APP_ICONS.len()
    );
    for (_, name) in APP_ICONS {
        let dest = embedder.embed(&format!("generated/{name}"), name, b"", "resume-generator");
        let _ = writeln!(
            icons,
            "    ({name:?}, include_bytes!({:?})),",
            dest.display().to_string()
        );
    }
    icons.push_str("];\n");
    fs::write(embedder.out_dir.join("app_icons.rs"), icons).expect("write app_icons.rs");

    embedder.finish();
}
