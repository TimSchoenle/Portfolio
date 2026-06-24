//! Generates localized, single-page resume PDFs from the shared portfolio data.
//!
//! Output (into `<out-dir>`, default `dist`):
//!   resume/Tim-Schönle-Resume.pdf      (en)
//!   resume/Tim-Schönle-Lebenslauf.pdf  (de)
//!   resume-fingerprint.json            (SHA-256 per file, shown on the contact card)
//!
//! Layout: an open header (name and title left, contact right) over a
//! gradient rule, a full-width professional summary, then a two-column body —
//! skills (sorted strongest first, grouped by category), education and
//! languages in a narrow left sidebar, reverse-chronological experience (each
//! role with a plain-text "Stack:" keyword line) in the main column. All
//! structural lines share one visual language: navy fading to a soft
//! blue-gray, drawn as interpolated stroke segments since genpdf exposes no
//! PDF shadings. Text is emitted in the order ATS parsers expect (identity →
//! summary → skills → experience → education), with standard section names
//! and consistent "Mon YYYY – Mon YYYY" ranges. Fonts are embedded
//! (Liberation Sans, SIL OFL, pre-subset to Latin-1) so text stays selectable
//! and machine-readable, including umlauts; the gradient lines are vector
//! strokes, invisible to text extraction.
//!
//! The single-page guarantee is dynamic and prefers readability over
//! shrinking: a binary search finds the largest scale ≥ 0.9 that fits; if
//! the content has grown too much, the two oldest roles are condensed to
//! their two strongest bullets before smaller scales are even considered.
//!
//! The pipeline is split across modules: [`style`] (colors and typography),
//! [`elements`] (custom gradient/bullet/divider elements), [`document`]
//! (assembly and fonts), [`fit`] (single-page fitting), [`translations`]
//! (the embedded i18n lookup) and [`metadata`] (UTF-16 info-dict fix-up).

mod document;
mod elements;
mod fit;
mod metadata;
mod style;
mod translations;

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::Path;

use portfolio_data::{CONFIG, I18N_DE, I18N_EN, RESUME_FILES};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use crate::document::load_fonts;
use crate::fit::fit_single_page;
use crate::metadata::{fix_metadata, hex};
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

    let fonts = load_fonts()?;
    let mut fingerprints: BTreeMap<String, String> = BTreeMap::new();

    for (lang, file_name) in RESUME_FILES {
        let json = match lang {
            "de" => I18N_DE,
            _ => I18N_EN,
        };
        let t = Translations::parse(json)?;
        let fitted = fit_single_page(&fonts, &t)
            .ok_or_else(|| format!("{file_name}: does not fit one page even condensed"))?;

        let title = format!("{} — {}", CONFIG.full_name, t.get("hero.jobTitle"));
        let bytes = fix_metadata(&fitted.bytes, &title)?;

        let path = resume_dir.join(file_name);
        fs::write(&path, &bytes)?;
        fingerprints.insert(file_name.to_string(), hex(&Sha256::digest(&bytes)));
        println!(
            "wrote {} ({} bytes, scale {:.2}, {})",
            path.display(),
            bytes.len(),
            fitted.scale,
            fitted.detail.describe()
        );
    }

    let manifest = serde_json::json!({
        "algorithm": "SHA-256",
        "generated_at": OffsetDateTime::now_utc().format(&Rfc3339)?,
        "files": fingerprints,
    });
    let manifest_path = Path::new(&out_dir).join("resume-fingerprint.json");
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)?;
    println!("wrote {}", manifest_path.display());

    Ok(())
}
