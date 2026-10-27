//! The resume fingerprint manifest the generator writes and the contact card shows.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::resume_file;

/// The digests of the generated resume PDFs.
///
/// The resume generator writes it as `resume-fingerprint.json`, and the web app embeds it at
/// build time and shows each digest on the contact card, so a downloaded resume can be checked
/// against it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResumeFingerprints {
    /// The hash algorithm of the digests, such as `SHA-256`.
    pub algorithm: String,
    /// The RFC 3339 timestamp the manifest was generated at.
    pub generated_at: String,
    /// The hex digest of each resume, keyed by its published file name.
    pub files: BTreeMap<String, String>,
}

impl ResumeFingerprints {
    /// Reports whether the manifest holds no digest, as in a build without generated resumes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }

    /// Returns the digest of the resume for the language code `lang`, if present.
    ///
    /// An unknown `lang` reads the default language's resume.
    pub fn digest_for(&self, lang: &str) -> Option<&str> {
        self.files.get(resume_file(lang)).map(String::as_str)
    }
}
