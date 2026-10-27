//! The fingerprints of the resume PDFs the server embeds, which the contact section links by.

use portfolio_data::ResumeFingerprints;

/// `resume-fingerprint.json`, embedded at compile time (empty default when no
/// resumes were produced).
const RESUME_FINGERPRINT_JSON: &str =
    include_str!(concat!(env!("OUT_DIR"), "/resume-fingerprint.json"));

/// Parses the embedded `resume-fingerprint.json`; `None` when no resumes were
/// generated (the embedded manifest is empty) or the manifest is malformed.
pub fn load_resume_fingerprints() -> Option<ResumeFingerprints> {
    let parsed = serde_json::from_str::<ResumeFingerprints>(RESUME_FINGERPRINT_JSON).ok()?;
    if parsed.is_empty() {
        None
    } else {
        Some(parsed)
    }
}
