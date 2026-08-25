//! Generates localized, single-page resume PDFs from the shared portfolio data.
//!
//! Output (into `<out-dir>`, default `dist`):
//!   resume/Tim-Schönle-Resume.pdf      (en)
//!   resume/Tim-Schönle-Lebenslauf.pdf  (de)
//!   resume-fingerprint.json            (SHA-256 per file, shown on the contact card)
//!   og-image.png                       (1200×630 social card, see [`og_image`])
//!
//! Layout: a two-column design rendered with Typst — a full-width header band
//! (QR code, name, availability) over a tinted sidebar (Contact, Skills,
//! Education, Languages) and a wide main column (Summary, Experience). The main
//! column is emitted *first* so a text extractor reads it in order (identity →
//! summary → experience → contact → skills) despite the sidebar showing on the
//! left. The output is a tagged, standard PDF 1.7 (RGB) with live `/URI` link
//! annotations.
//!
//! # Usage
//!
//! ```text
//! resume-generator [<out-dir>] [--photo <file>]
//! ```
//!
//! `--photo` (or `PORTFOLIO_RESUME__PHOTO_FILE`) supplies the application photo
//! the German sheet puts at the top of its sidebar. It is optional and nothing
//! renders in its place when it is absent, so the default output is the same
//! photo-less pair of sheets it has always been. The English sheet never renders
//! it at all.
//!
//! The file is a **build input, never a committed asset** — a portrait is
//! personal data that belongs neither in the repository nor in an image layer.
//! The image build takes it as a `BuildKit` secret that exists only for the step
//! that reads it:
//!
//! ```text
//! docker build --secret id=resume_photo,src=./photo.jpg .
//! ```
//!
//! JPEG, PNG and WebP are accepted, and the format is read from the file's own
//! header, so a secret mounted under a name with no extension works.
//!
//! The pipeline is split across modules: [`style`] (design tokens), [`template`]
//! (the `.typ` document generator), [`qr`] (the header's vector QR code),
//! [`world`] (the embedded-font Typst `World` and PDF export), [`fit`]
//! (single-page fitting) and [`translations`] (the embedded i18n lookup).

mod fit;
mod og_image;
mod qr;
mod style;
mod template;
mod translations;
mod world;

use std::collections::BTreeMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use portfolio_data::{CONFIG, RESUME_FILES, ResumeFingerprints};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use typst::foundations::Bytes;

use crate::fit::fit_single_page;
use crate::template::Photo;
use crate::translations::Translations;
use crate::world::Asset;

/// Environment variable the photo path may arrive through instead of `--photo`,
/// named for the `PORTFOLIO_` convention the rest of the estate uses.
const PHOTO_ENV: &str = "PORTFOLIO_RESUME__PHOTO_FILE";

/// The image formats the photo may arrive in, as `(magic bytes, extension)`.
///
/// The format is read from the file's own header rather than from its name,
/// because the path this arrives on is usually not a name at all — a `BuildKit`
/// secret mounts as `/run/secrets/<id>` with no extension. Typst picks its
/// decoder from the extension, so the sniffed one is what the asset is
/// registered under.
const PHOTO_FORMATS: [(&[u8], &str); 3] = [
    (b"\xFF\xD8\xFF", "jpg"),
    (b"\x89PNG\r\n\x1A\n", "png"),
    // WebP is `RIFF....WEBP`; the four size bytes in between are skipped by
    // matching the two halves separately.
    (b"RIFF", "webp"),
];

fn main() {
    if let Err(err) = run() {
        eprintln!("resume-generator: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let (out_dir, photo_path) = parse_args(std::env::args().skip(1))?;
    let resume_dir = Path::new(&out_dir).join("resume");
    fs::create_dir_all(&resume_dir)?;

    let assets = match &photo_path {
        Some(path) => vec![load_photo(path)?],
        None => Vec::new(),
    };
    let photo = assets.first().map(|asset| Photo {
        file_name: &asset.name,
    });
    if let Some(path) = &photo_path {
        println!(
            "using application photo {} (German sheet only)",
            path.display()
        );
    }

    let mut fingerprints: BTreeMap<String, String> = BTreeMap::new();

    for (lang, file_name) in RESUME_FILES {
        let json = match lang {
            "de" => portfolio_data::I18N_DE,
            _ => portfolio_data::I18N_EN,
        };
        let t = Translations::parse(json)?;
        let fitted = fit_single_page(&t, lang, photo, &assets)
            .map_err(|err| format!("{file_name}: {err}"))?;

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

/// Splits the command line into the output directory and the optional photo
/// path, falling back to [`PHOTO_ENV`] when `--photo` is absent.
///
/// The first bare word is the output directory; `dist` when there is none.
fn parse_args(mut args: impl Iterator<Item = String>) -> Result<(String, Option<PathBuf>), String> {
    let mut out_dir = None;
    let mut photo = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--photo" => {
                photo = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| "--photo needs a file path".to_string())?,
                ));
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown option {other}"));
            }
            other if out_dir.is_none() => out_dir = Some(other.to_string()),
            other => return Err(format!("unexpected argument {other}")),
        }
    }
    let photo = photo.or_else(|| std::env::var_os(PHOTO_ENV).map(PathBuf::from));
    Ok((out_dir.unwrap_or_else(|| "dist".to_string()), photo))
}

/// Reads the application photo into an in-memory [`Asset`], named for the
/// format its own header says it is.
fn load_photo(path: &Path) -> Result<Asset, String> {
    let bytes = fs::read(path).map_err(|err| format!("{}: {err}", path.display()))?;
    let extension = photo_format(&bytes).ok_or_else(|| {
        format!(
            "{}: the photo has to be a JPEG, PNG or WebP image",
            path.display(),
        )
    })?;
    Ok(Asset {
        name: format!("photo.{extension}"),
        bytes: Bytes::new(bytes),
    })
}

/// The extension for `bytes`, read from its file header, or `None` when it is
/// not one of [`PHOTO_FORMATS`].
fn photo_format(bytes: &[u8]) -> Option<&'static str> {
    PHOTO_FORMATS
        .into_iter()
        .find(|(magic, extension)| {
            bytes.starts_with(magic)
                // RIFF is a container: only the `WEBP` form of it is an image.
                && (*extension != "webp" || bytes.get(8..12) == Some(b"WEBP"))
        })
        .map(|(_, extension)| extension)
}

/// Lowercase hex encoding of a byte slice (for SHA-256 digests).
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(items: &[&str]) -> Result<(String, Option<PathBuf>), String> {
        parse_args(items.iter().map(|s| (*s).to_string()))
    }

    #[test]
    fn the_output_directory_defaults_and_the_photo_is_optional() {
        assert_eq!(args(&[]).expect("no arguments"), ("dist".to_string(), None));
        assert_eq!(
            args(&["out"]).expect("one positional"),
            ("out".to_string(), None),
        );
    }

    #[test]
    fn the_photo_flag_is_read_in_either_position() {
        let expected = ("out".to_string(), Some(PathBuf::from("a.jpg")));
        assert_eq!(
            args(&["out", "--photo", "a.jpg"]).expect("trailing"),
            expected
        );
        assert_eq!(
            args(&["--photo", "a.jpg", "out"]).expect("leading"),
            expected
        );
    }

    #[test]
    fn a_malformed_command_line_is_refused_rather_than_guessed_at() {
        assert!(args(&["--photo"]).is_err());
        assert!(args(&["--nope"]).is_err());
        assert!(args(&["one", "two"]).is_err());
    }

    #[test]
    fn the_photo_format_comes_from_the_file_header() {
        assert_eq!(photo_format(b"\xFF\xD8\xFF\xE0rest"), Some("jpg"));
        assert_eq!(photo_format(b"\x89PNG\r\n\x1A\nrest"), Some("png"));
        assert_eq!(photo_format(b"RIFF\0\0\0\0WEBPVP8 "), Some("webp"));
    }

    #[test]
    fn a_file_that_is_not_an_image_is_refused() {
        // A RIFF container that is not WebP (a WAV, say) is not an image.
        assert_eq!(photo_format(b"RIFF\0\0\0\0WAVEfmt "), None);
        assert_eq!(photo_format(b"not an image at all"), None);
        assert_eq!(photo_format(b""), None);
    }
}
