//! Optional Sigstore signing of the generated resume PDFs via the external
//! [`pdf-sign`](https://github.com/0x77dev/pdf-sign) tool (keyless OIDC).
//!
//! Signing is opt-in and only happens when an OIDC identity token is available
//! in the environment (`SIGSTORE_IDENTITY_TOKEN`), which is the case on CI where
//! the release workflow supplies a pre-obtained Sigstore (Dex) email-identity
//! token. Local dev builds leave the variable unset, so signing is skipped
//! entirely and no network access or `pdf-sign` binary is required.
//!
//! The signature is appended after the PDF's `%%EOF`, so the signed file is a
//! superset of the original and stays a valid PDF. Because the file changes, the
//! generator hashes the *signed* bytes, keeping the fingerprint shown on the site
//! in sync with the actual download.

use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

use portfolio_data::ResumeSignature;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// Sigstore (Dex) OIDC issuer that mints the email-based identity the resume
/// PDFs are signed with; recorded for display alongside the fingerprint.
const SIGSTORE_OIDC_ISSUER: &str = "https://oauth2.sigstore.dev/auth";

/// Returns the OIDC identity token when Sigstore signing is enabled, else
/// `None` (signing is skipped — local builds, no token configured).
pub fn identity_token() -> Option<String> {
    token_from_value(std::env::var("SIGSTORE_IDENTITY_TOKEN").ok())
}

/// Decides whether a raw environment value enables signing: a present,
/// non-blank token enables it; an absent, empty, or whitespace-only value does
/// not. Kept pure (no env access) so the signing gate can be unit-tested.
fn token_from_value(value: Option<String>) -> Option<String> {
    value.filter(|t| !t.trim().is_empty())
}

/// Signs `path` in place with `pdf-sign` (Sigstore backend, keyless OIDC) using
/// the given identity `token`, returning the signature metadata to record in the
/// fingerprint manifest. On success the file at `path` is replaced by its signed
/// counterpart.
pub fn sign_pdf(path: &Path, token: &str) -> Result<ResumeSignature, Box<dyn Error>> {
    // `pdf-sign` writes a separate output file; sign into a sibling and swap it in
    // so a failure never leaves a half-written PDF in place.
    let signed = path.with_extension("signed.pdf");

    let output = Command::new("pdf-sign")
        .arg("sign")
        .args(["--backend", "sigstore"])
        .args(["--identity-token", token])
        .arg("--output")
        .arg(&signed)
        .arg(path)
        .output()
        .map_err(|err| format!("failed to run pdf-sign (is it installed?): {err}"))?;

    if !output.status.success() {
        let _ = fs::remove_file(&signed);
        return Err(format!(
            "pdf-sign exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into());
    }

    fs::rename(&signed, path)?;

    signature_metadata()
}

/// Builds the [`ResumeSignature`] recorded in the fingerprint manifest for a
/// freshly signed PDF (Sigstore backend, current signer identity and issuer,
/// RFC 3339 timestamp).
fn signature_metadata() -> Result<ResumeSignature, Box<dyn Error>> {
    Ok(ResumeSignature {
        backend: "sigstore".to_string(),
        identity: signer_identity(),
        issuer: SIGSTORE_OIDC_ISSUER.to_string(),
        rekor_log_url: None,
        signed_at: OffsetDateTime::now_utc().format(&Rfc3339)?,
    })
}

/// The email identity bound to the signing certificate, supplied by the release
/// workflow via `SIGSTORE_IDENTITY_EMAIL` (the contact email the Sigstore token
/// is issued for), which is exactly what `pdf-sign verify --certificate-identity`
/// expects.
fn signer_identity() -> String {
    format_signer_identity(std::env::var("SIGSTORE_IDENTITY_EMAIL").ok())
}

/// Formats the signer identity from the configured contact email, falling back
/// to `"unknown"` when it is absent or empty. Kept pure (no env access) so the
/// formatting can be unit-tested.
fn format_signer_identity(email: Option<String>) -> String {
    match email {
        Some(email) if !email.trim().is_empty() => email,
        _ => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_present_and_non_blank_enables_signing() {
        assert_eq!(
            token_from_value(Some("oidc-token".to_string())),
            Some("oidc-token".to_string())
        );
    }

    #[test]
    fn token_absent_blank_or_empty_skips_signing() {
        assert_eq!(token_from_value(None), None);
        assert_eq!(token_from_value(Some(String::new())), None);
        assert_eq!(token_from_value(Some("   \t\n".to_string())), None);
    }

    #[test]
    fn signer_identity_uses_the_configured_email() {
        assert_eq!(
            format_signer_identity(Some("contact@tim-schoenle.de".to_string())),
            "contact@tim-schoenle.de"
        );
    }

    #[test]
    fn signer_identity_is_unknown_without_an_email() {
        assert_eq!(format_signer_identity(None), "unknown");
        assert_eq!(format_signer_identity(Some(String::new())), "unknown");
        assert_eq!(format_signer_identity(Some("   \t\n".to_string())), "unknown");
    }

    #[test]
    fn signature_metadata_records_the_sigstore_backend_and_issuer() {
        let sig = signature_metadata().expect("signature metadata builds");
        assert_eq!(sig.backend, "sigstore");
        assert_eq!(sig.issuer, SIGSTORE_OIDC_ISSUER);
        assert!(sig.rekor_log_url.is_none());
        // The timestamp must be a valid RFC 3339 instant.
        assert!(OffsetDateTime::parse(&sig.signed_at, &Rfc3339).is_ok());
    }
}
