//! What the served `Content-Security-Policy` has to make room for.
//!
//! The policy itself is not configurable. The SSR server builds it with `csp-shell`, and a site
//! that could be handed arbitrary directives through a `ConfigMap` would have no policy at all.
//! What is configurable is the small set of facts this crate cannot know: whether the inline
//! scripts in the served document are covered by their hashes, and which Cloudflare products are
//! switched on in front of the origin.

use core::fmt;

use serde::Deserialize;

/// How the SSR server builds the `Content-Security-Policy` it sends.
///
/// Every field defaults to what this deployment runs, so an empty or absent `[csp]` block produces
/// the policy the site is meant to serve. The settings cover the two situations a redeploy is too
/// slow for: an inline script the scanner does not see, and a Cloudflare product switched on or
/// off in the dashboard.
// No `deny_unknown_fields`, for the reasons given on `AssetsConfig`: closing a `#[config(nested)]`
// block narrows what the loader accepts and publishes nothing to the contract.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(
    feature = "config-schema",
    derive(serde::Serialize, terrace_config::schema::Describe)
)]
pub struct CspConfig {
    /// Whether the server admits inline `<script>` by hash, in place of `'unsafe-inline'`.
    ///
    /// On, the default, the server hashes the response body it is about to send, so the only
    /// inline scripts that run are the ones it rendered. `'unsafe-inline'` admits an injected
    /// `<script>` as readily as the Dioxus hydration bootstrap.
    ///
    /// Off restores `'unsafe-inline'` with a restart. Use it when the browser refuses an inline
    /// script the scanner does not see, which shows as a blank page and a console message. Known
    /// cases are a development server that injects a live-reload script downstream of this
    /// process, and a renderer change that emits script this server never sees.
    ///
    /// A policy carrying any hash or nonce makes `'unsafe-inline'` inert, so turning this off also
    /// requires `csp.cloudflare.script_nonce = false`. [`validate`](Self::validate) refuses the
    /// boot otherwise.
    #[serde(default = "CspConfig::default_hash_inline_scripts")]
    pub hash_inline_scripts: bool,

    /// The Cloudflare products in front of this origin that the policy has to make room for.
    #[cfg_attr(feature = "config-schema", config(nested))]
    #[serde(default)]
    pub cloudflare: CloudflareConfig,
}

impl CspConfig {
    /// Returns the default of [`Self::hash_inline_scripts`], on.
    const fn default_hash_inline_scripts() -> bool {
        true
    }

    /// Checks that the block does not describe a policy refusing this site's own scripts.
    ///
    /// The server calls it at boot, before the listener exists, so the container fails to start
    /// instead of serving every visitor a blank page.
    ///
    /// # Errors
    ///
    /// [`CspConfigError::NonceWithoutInlineHashes`] when a nonce is reserved while inline scripts
    /// are admitted by `'unsafe-inline'`.
    pub const fn validate(&self) -> Result<(), CspConfigError> {
        if self.cloudflare.script_nonce && !self.hash_inline_scripts {
            return Err(CspConfigError::NonceWithoutInlineHashes);
        }
        Ok(())
    }
}

impl Default for CspConfig {
    fn default() -> Self {
        Self {
            hash_inline_scripts: Self::default_hash_inline_scripts(),
            cloudflare: CloudflareConfig::default(),
        }
    }
}

/// The Cloudflare products this origin sits behind, as the policy has to see them.
///
/// Each field is one `csp-shell` preset. A preset knows which origins a product loads from and
/// which directives they have to appear in: admitting Turnstile's script without its frame
/// renders an empty box.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(
    feature = "config-schema",
    derive(serde::Serialize, terrace_config::schema::Describe)
)]
pub struct CloudflareConfig {
    /// Whether `script-src` reserves a per-response nonce for the script Cloudflare injects at the
    /// edge.
    ///
    /// On by default, because this site is delivered through a Cloudflare Tunnel with the bot
    /// products active; the `__cf_bm` and `cf_clearance` cookies are theirs. Those products inject
    /// an inline `<script>` after the HTML leaves this process, so no hash the server computes can
    /// cover it, and without the nonce the detection silently never runs. Cloudflare reads the
    /// nonce from the `Content-Security-Policy` response header and copies it onto what it
    /// injects.
    ///
    /// The server sends `Cache-Control: no-cache` on every document, so a cache must revalidate a
    /// stored copy with this origin before each reuse. A document carries no validator, so that
    /// revalidation is a full render with a fresh nonce. **No Cloudflare Cache Rule may cache the
    /// documents**: a "Cache Everything" rule overrides that header at the edge, and nothing
    /// inside this process can see it.
    #[serde(default = "CloudflareConfig::default_script_nonce")]
    pub script_nonce: bool,

    /// Whether `script-src` and `frame-src` admit `https://challenges.cloudflare.com`, for a
    /// Turnstile widget.
    ///
    /// Off, because this site has no form behind a challenge. A managed-challenge interstitial
    /// needs nothing here: it is a Cloudflare-served document carrying its own policy.
    #[serde(default)]
    pub turnstile: bool,

    /// Whether the policy admits the Cloudflare Web Analytics beacon and the endpoint it reports
    /// to.
    ///
    /// Off, because this site measures nothing, which the privacy page states. It covers the
    /// manual snippet only; the automatic edge injection is an inline script and needs
    /// `script_nonce`.
    #[serde(default)]
    pub web_analytics: bool,
}

impl CloudflareConfig {
    /// Returns the default of [`Self::script_nonce`], on.
    const fn default_script_nonce() -> bool {
        true
    }
}

impl Default for CloudflareConfig {
    fn default() -> Self {
        Self {
            script_nonce: Self::default_script_nonce(),
            turnstile: false,
            web_analytics: false,
        }
    }
}

/// The reason a `[csp]` block cannot be served.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CspConfigError {
    /// `csp.cloudflare.script_nonce` is on while `csp.hash_inline_scripts` is off.
    NonceWithoutInlineHashes,
}

impl fmt::Display for CspConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonceWithoutInlineHashes => f.write_str(
                "csp.cloudflare.script_nonce reserves a nonce in script-src, and a browser \
                 ignores 'unsafe-inline' as soon as the policy carries any nonce or hash — so \
                 with csp.hash_inline_scripts = false this server's own inline scripts would be \
                 refused and every page would render blank. Set csp.hash_inline_scripts = true, \
                 or turn the nonce off",
            ),
        }
    }
}

impl core::error::Error for CspConfigError {}

#[cfg(test)]
mod tests {
    use super::{CloudflareConfig, CspConfig, CspConfigError};

    /// The defaults are the deployment: hashes on, the Cloudflare nonce on, and no origin
    /// admitted for a product this site does not use. A preset switched on by default would
    /// widen the policy for a script nobody loads.
    #[test]
    fn the_defaults_describe_this_deployment() {
        let config = CspConfig::default();
        assert!(config.hash_inline_scripts);
        assert!(config.cloudflare.script_nonce);
        assert!(!config.cloudflare.turnstile);
        assert!(!config.cloudflare.web_analytics);
        assert_eq!(config.validate(), Ok(()));
    }

    /// The escape hatch is usable on its own: dropping to `'unsafe-inline'` also has to drop the
    /// nonce, or the policy it produces refuses the scripts it exists to permit.
    #[test]
    fn a_nonce_without_inline_hashes_is_refused() {
        let config = CspConfig {
            hash_inline_scripts: false,
            cloudflare: CloudflareConfig::default(),
        };
        assert_eq!(
            config.validate(),
            Err(CspConfigError::NonceWithoutInlineHashes)
        );
    }

    #[test]
    fn dropping_the_nonce_makes_the_escape_hatch_valid() {
        let config = CspConfig {
            hash_inline_scripts: false,
            cloudflare: CloudflareConfig {
                script_nonce: false,
                ..CloudflareConfig::default()
            },
        };
        assert_eq!(config.validate(), Ok(()));
    }

    /// The error names both keys, because the operator who reads it has to change one of them.
    #[test]
    fn the_refusal_names_the_two_keys_that_conflict() {
        let message = CspConfigError::NonceWithoutInlineHashes.to_string();
        assert!(message.contains("csp.cloudflare.script_nonce"));
        assert!(message.contains("csp.hash_inline_scripts"));
    }
}
