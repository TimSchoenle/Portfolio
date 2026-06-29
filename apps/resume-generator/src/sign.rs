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
    std::env::var("SIGSTORE_IDENTITY_TOKEN")
        .ok()
        .filter(|t| !t.trim().is_empty())
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
/// `https://github.com/owner/repo/.github/workflows/ci.yaml@refs/heads/main`,
/// which is exactly what `pdf-sign verify --certificate-identity` expects.
fn signer_identity() -> String {
    match (
        std::env::var("GITHUB_SERVER_URL"),
        std::env::var("GITHUB_WORKFLOW_REF"),
    ) {
        (Ok(server), Ok(workflow_ref)) if !workflow_ref.is_empty() => {
            format!("{}/{}", server.trim_end_matches('/'), workflow_ref)
        }
        _ => "unknown".to_string(),
    }
}
