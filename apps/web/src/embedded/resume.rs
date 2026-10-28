//! The fingerprints of the resume PDFs the server embeds, which the contact section links by.

use portfolio_data::ResumeFingerprints;

/// `resume-fingerprint.json`, embedded at compile time (empty default when no
/// resumes were produced).
const RESUME_FINGERPRINT_JSON: &str =
    include_str!(concat!(env!("OUT_DIR"), "/resume-fingerprint.json"));

/// Parses the embedded `resume-fingerprint.json`; `None` when no resumes were
/// generated (the embedded manifest is empty) or the manifest is malformed.
pub fn load_resume_fingerprints() -> Option<ResumeFingerprints> {
    parse(RESUME_FINGERPRINT_JSON)
}

/// Parses a fingerprint manifest; `None` when it lists no file or does not parse.
fn parse(json: &str) -> Option<ResumeFingerprints> {
    serde_json::from_str::<ResumeFingerprints>(json)
        .ok()
        .filter(|parsed| !parsed.is_empty())
}

#[cfg(test)]
mod tests {
    use super::parse;

    #[test]
    fn a_manifest_with_files_parses() {
        let json = r#"{"algorithm":"SHA-256","generated_at":"2026-01-01T00:00:00Z","files":{"a.pdf":"00ff"}}"#;
        let parsed = parse(json).expect("a manifest with one file");
        assert_eq!(parsed.files.get("a.pdf").map(String::as_str), Some("00ff"));
    }

    #[test]
    fn the_empty_default_is_none() {
        assert_eq!(
            parse(r#"{"algorithm":"","generated_at":"","files":{}}"#),
            None
        );
    }

    #[test]
    fn a_malformed_manifest_is_none() {
        assert_eq!(parse("{"), None);
    }
}
