//! Generates localized, single-page resume PDFs from the shared portfolio data.
//!
//! Output (into `<out-dir>`, default `dist`):
//!   resume/Tim-Schönle-Resume.pdf      (en)
//!   resume/Tim-Schönle-Lebenslauf.pdf  (de)
//!   resume-fingerprint.json            (SHA-256 per file, shown on the contact card)
//!
//! Layout: a two-column design rendered with Typst — a full-width header band
//! over a tinted sidebar (Contact, Skills, Education, Languages) and a wide main
//! column (Summary, Experience). The main column is emitted *first* so a text
//! extractor reads it in order (identity → summary → experience → contact →
//! skills) despite the sidebar showing on the left. The output is a tagged,
//! standard PDF 1.7 (RGB) with live `/URI` link annotations.
//!
//! The pipeline is split across modules: [`style`] (design tokens), [`template`]
//! (the `.typ` document generator), [`world`] (the embedded-font Typst `World`
//! and PDF export), [`fit`] (single-page fitting) and [`translations`] (the
//! embedded i18n lookup).

mod fit;
mod sign;
mod style;
mod template;
mod translations;
mod world;

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use portfolio_data::{RESUME_FILES, ResumeFingerprints, ResumeSignature};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::fit::fit_single_page;
use crate::translations::Translations;

fn main() {
    if let Err(err) = run() {
        eprintln!("resume-generator: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let out_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "dist".to_string());
    let resume_dir = Path::new(&out_dir).join("resume");
    fs::create_dir_all(&resume_dir)?;

    let mut fingerprints: BTreeMap<String, String> = BTreeMap::new();
    let mut signatures: BTreeMap<String, ResumeSignature> = BTreeMap::new();

    // When an OIDC identity token is present (CI), each PDF is signed via
    // `pdf-sign` (Sigstore keyless) right after it is written, before hashing.
    let signing_token = sign::identity_token();

    for (lang, file_name) in RESUME_FILES {
        let json = match lang {
            "de" => portfolio_data::I18N_DE,
            _ => portfolio_data::I18N_EN,
        };
        let t = Translations::parse(json)?;
        let fitted =
            fit_single_page(&t, lang).map_err(|err| format!("{file_name}: {err}"))?;

        let path = resume_dir.join(file_name);
        fs::write(&path, &fitted.bytes)?;
        println!(
            "wrote {} ({} bytes, scale {:.2}, {} density, {})",
            path.display(),
            fitted.bytes.len(),
            fitted.scale,
            if fitted.dense { "compact" } else { "comfortable" },
            fitted.detail.describe()
        );

        // Optionally append a Sigstore signature; this rewrites the file, so the
        // digest is taken from the final bytes on disk to match the download.
        if let Some(token) = signing_token.as_deref() {
            let signature = sign::sign_pdf(&path, token)
                .map_err(|err| format!("{file_name}: signing failed: {err}"))?;
            println!("signed {} as {}", path.display(), signature.identity);
            signatures.insert(file_name.to_string(), signature);
        }

        let bytes = fs::read(&path)?;
        fingerprints.insert(file_name.to_string(), hex(&Sha256::digest(&bytes)));
    }

    let manifest = ResumeFingerprints {
        algorithm: "SHA-256".to_string(),
        generated_at: OffsetDateTime::now_utc().format(&Rfc3339)?,
        files: fingerprints,
        signatures,
    };
    let manifest_path = Path::new(&out_dir).join("resume-fingerprint.json");
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;
    println!("wrote {}", manifest_path.display());

    Ok(())
}

/// Lowercase hex encoding of a byte slice (for SHA-256 digests).
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
