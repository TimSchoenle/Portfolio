//! Generates localized, single-page resume PDFs from the shared portfolio data.
//!
//! Output (into `<out-dir>`, default `dist`):
//!   resume/Tim-Schönle-Resume.pdf      (en)
//!   resume/Tim-Schönle-Lebenslauf.pdf  (de)
//!   resume-fingerprint.json            (SHA-256 per file, shown on the contact card)
//!   og-image.png                       (1200×630 social card, see [`og_image`])
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
mod og_image;
mod style;
mod template;
mod translations;
mod world;

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use portfolio_data::{CONFIG, RESUME_FILES, ResumeFingerprints};
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

    for (lang, file_name) in RESUME_FILES {
        let json = match lang {
            "de" => portfolio_data::I18N_DE,
            _ => portfolio_data::I18N_EN,
        };
        let t = Translations::parse(json)?;
        let fitted = fit_single_page(&t, lang).map_err(|err| format!("{file_name}: {err}"))?;

        let path = resume_dir.join(file_name);
        fs::write(&path, &fitted.bytes)?;
        println!(
            "wrote {} ({} bytes, scale {:.2}, {} density, {})",
            path.display(),
            fitted.bytes.len(),
            fitted.scale,
            if fitted.dense {
                "compact"
            } else {
                "comfortable"
            },
            fitted.detail.describe()
        );

        let bytes = fs::read(&path)?;
        fingerprints.insert(file_name.to_string(), hex(&Sha256::digest(&bytes)));
    }

    let manifest = ResumeFingerprints {
        algorithm: "SHA-256".to_string(),
        generated_at: OffsetDateTime::now_utc().format(&Rfc3339)?,
        files: fingerprints,
    };
    let manifest_path = Path::new(&out_dir).join("resume-fingerprint.json");
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;
    println!("wrote {}", manifest_path.display());

    // The social card, alongside the resumes: another artifact derived from the
    // shared data here and embedded into the web binary by its `build.rs`. Paired
    // with `CONFIG` rather than the translations so it says exactly what the
    // `og:title`/`og:description` beside it say.
    let og_path = Path::new(&out_dir).join(portfolio_data::OG_IMAGE_FILE);
    let og_bytes = og_image::render(CONFIG.job_title, CONFIG.description)?;
    fs::write(&og_path, &og_bytes)?;
    let (width, height) = portfolio_data::OG_IMAGE_SIZE;
    println!(
        "wrote {} ({} bytes, {width}×{height})",
        og_path.display(),
        og_bytes.len()
    );

    Ok(())
}

/// Lowercase hex encoding of a byte slice (for SHA-256 digests).
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
