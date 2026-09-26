//! The resume fingerprint manifest the generator writes and the contact card shows.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::resume_file;

/// SHA-256 checksums of the generated resume PDFs.
///
/// Written by the resume generator as `resume-fingerprint.json` and embedded
/// into the frontend at build time, where it is shown on the contact card so a
/// downloaded resume can be verified against its published digest.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResumeFingerprints {
    /// Hash algorithm used for the digests (e.g. "SHA-256").
    pub algorithm: String,
    /// RFC 3339 timestamp of when the manifest was generated.
    pub generated_at: String,
    /// Resume file name (e.g. "Tim-Schönle-Resume.pdf") -> hex digest.
    pub files: BTreeMap<String, String>,
}

impl ResumeFingerprints {
    /// `true` when no resume digests are present (e.g. dev builds without
    /// generated resumes).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Digest of the resume for the given language, if present.
    pub fn digest_for(&self, lang: &str) -> Option<&str> {
        self.files.get(resume_file(lang)).map(String::as_str)
    }
}
