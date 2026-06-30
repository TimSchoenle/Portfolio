//! Optional Sigstore signing of the generated resume PDFs via the external
//! [`pdf-sign`](https://github.com/0x77dev/pdf-sign) tool (keyless OIDC).
//!
//! Signing is opt-in and only happens when an OIDC identity token is available
//! in the environment (`SIGSTORE_IDENTITY_TOKEN`), which is the case on CI after
//! the workflow mints a GitHub Actions token with the `sigstore` audience. Local
//! dev builds leave the variable unset, so signing is skipped entirely and no
//! network access or `pdf-sign` binary is required.
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

/// Default issuer for GitHub Actions OIDC tokens; recorded for display when the
/// build runs on GitHub Actions.
const GITHUB_OIDC_ISSUER: &str = "https://token.actions.githubusercontent.com";

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
        issuer: GITHUB_OIDC_ISSUER.to_string(),
        rekor_log_url: None,
        signed_at: OffsetDateTime::now_utc().format(&Rfc3339)?,
    })
}

/// The OIDC identity bound to the signing certificate. On GitHub Actions this is
/// `{GITHUB_SERVER_URL}/{GITHUB_WORKFLOW_REF}`, e.g.
/// `https://github.com/owner/repo/.github/workflows/release-please.yaml@refs/heads/main`,
/// which is exactly what `pdf-sign verify --certificate-identity` expects.
fn signer_identity() -> String {
    format_signer_identity(
        std::env::var("GITHUB_SERVER_URL").ok(),
        std::env::var("GITHUB_WORKFLOW_REF").ok(),
    )
}

/// Formats the signer identity from the GitHub Actions environment as
/// `{server}/{workflow_ref}` (with any trailing slash on the server trimmed),
/// falling back to `"unknown"` when the workflow ref is absent or empty. Kept
/// pure (no env access) so the formatting can be unit-tested.
fn format_signer_identity(server: Option<String>, workflow_ref: Option<String>) -> String {
    match (server, workflow_ref) {
        (Some(server), Some(workflow_ref)) if !workflow_ref.is_empty() => {
            format!("{}/{}", server.trim_end_matches('/'), workflow_ref)
        }
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
    fn signer_identity_joins_server_and_workflow_ref() {
        assert_eq!(
            format_signer_identity(
                Some("https://github.com".to_string()),
                Some(".github/workflows/release-please.yaml@refs/heads/main".to_string()),
            ),
            "https://github.com/.github/workflows/release-please.yaml@refs/heads/main"
        );
    }

    #[test]
    fn signer_identity_trims_trailing_slash_on_server() {
        assert_eq!(
            format_signer_identity(
                Some("https://github.com/".to_string()),
                Some("wf@ref".to_string()),
            ),
            "https://github.com/wf@ref"
        );
    }

    #[test]
    fn signer_identity_is_unknown_without_a_workflow_ref() {
        assert_eq!(
            format_signer_identity(Some("https://github.com".to_string()), None),
            "unknown"
        );
        assert_eq!(
            format_signer_identity(
                Some("https://github.com".to_string()),
                Some(String::new()),
            ),
            "unknown"
        );
        assert_eq!(format_signer_identity(None, None), "unknown");
    }

    #[test]
    fn signature_metadata_records_the_sigstore_backend_and_issuer() {
        let sig = signature_metadata().expect("signature metadata builds");
        assert_eq!(sig.backend, "sigstore");
        assert_eq!(sig.issuer, GITHUB_OIDC_ISSUER);
        assert!(sig.rekor_log_url.is_none());
        // The timestamp must be a valid RFC 3339 instant.
        assert!(OffsetDateTime::parse(&sig.signed_at, &Rfc3339).is_ok());
    }
}
