//! What the served `Content-Security-Policy` has to make room for.
//!
//! The policy itself is not configurable — it is built in the SSR server from `csp-shell`, and a
//! site that could be handed arbitrary directives through a `ConfigMap` would have no policy at
//! all. What *is* configurable is the small set of facts this crate cannot know: whether the
//! inline scripts in the served document are covered by their hashes, and which Cloudflare
//! products are switched on in front of the origin.

use core::fmt;

use serde::Deserialize;

/// How the SSR server builds the `Content-Security-Policy` it sends.
///
/// Every field defaults to what this deployment actually runs, so an empty `[csp]` block — or no
/// block at all — produces the policy the site is meant to serve. The knobs exist for the two
/// situations a redeploy is too slow for: an inline script the scanner does not see, and a
/// Cloudflare product switched on or off in the dashboard.
// No `deny_unknown_fields`, for the reasons given on `AssetsConfig`: closing a `#[config(nested)]`
// block narrows what the loader accepts and publishes nothing to the contract.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(
    feature = "config-schema",
    derive(serde::Serialize, terrace_config::schema::Describe)
)]
pub struct CspConfig {
    /// Hash every inline `<script>` in the document being served instead of admitting all inline
    /// script with `'unsafe-inline'`.
    ///
    /// On — the default — the server hashes the response body it is about to send, so the only
    /// inline scripts that run are the ones it rendered itself. That is the whole point of the
    /// exercise: `'unsafe-inline'` admits an injected `<script>` just as readily as the Dioxus
    /// hydration bootstrap.
    ///
    /// Off is the escape hatch, and it exists because the failure mode is silent: an inline
    /// script the scanner does not see is refused by the browser, and the evidence is a blank
    /// page and a console message nobody is watching. Turning it off restores `'unsafe-inline'`
    /// through an environment variable and a restart rather than a redeploy. Known cases: a
    /// development server that injects its own live-reload script downstream of this process,
    /// and any future renderer change that emits script this server never sees.
    ///
    /// The two settings are mutually exclusive in the browser, not merely different: a policy
    /// carrying any hash or nonce makes `'unsafe-inline'` inert, which is why there is one switch
    /// here rather than two independent ones.
    #[serde(default = "CspConfig::default_hash_inline_scripts")]
    pub hash_inline_scripts: bool,

    /// The Cloudflare products in front of this origin that the policy has to make room for.
    #[cfg_attr(feature = "config-schema", config(nested))]
    #[serde(default)]
    pub cloudflare: CloudflareConfig,
}

impl CspConfig {
    /// Hashing is the default; see the field.
    const fn default_hash_inline_scripts() -> bool {
        true
    }

    /// Reject a combination of values that would serve a policy refusing this site's own scripts.
    ///
    /// Called at boot, before the listener exists, so the container fails to start rather than
    /// serving every visitor a blank page.
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
/// Each field is one `csp-shell` preset. A preset carries the part that is easy to get wrong and
/// silent when it is: not only which origins a product loads from, but which directives they have
/// to appear in — admitting Turnstile's script without its frame renders an empty box.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(
    feature = "config-schema",
    derive(serde::Serialize, terrace_config::schema::Describe)
)]
pub struct CloudflareConfig {
    /// Reserve a per-response nonce in `script-src` for the script Cloudflare injects at the edge.
    ///
    /// On by default, because this site is delivered through a Cloudflare Tunnel with the bot
    /// products active — the privacy page lists `_cf_bm`, `cf_clearance` and the `cf_chl_rc_*`
    /// challenge cookies, which is the observable half of the same feature. Those products inject
    /// an inline `<script>` into the HTML *after* it leaves this process, so no hash this server
    /// computes can cover it; `script-src` refuses it and the detection silently never runs.
    /// Cloudflare's documented answer is to parse the `Content-Security-Policy` response header
    /// and copy the nonce onto what it injects, so nothing has to be stamped into the document.
    ///
    /// It carries one obligation the server discharges (`Cache-Control: no-cache` on every
    /// document, so a nonce is never shared between readers) and one it cannot: **no Cloudflare
    /// Cache Rule may cache the shell**. A "Cache Everything" rule overrides the origin's
    /// `Cache-Control`, satisfying the obligation here and violating it at the edge, and nothing
    /// inside this process can see that.
    #[serde(default = "CloudflareConfig::default_script_nonce")]
    pub script_nonce: bool,

    /// Admit `https://challenges.cloudflare.com` in `script-src` and `frame-src`, for a Turnstile
    /// widget.
    ///
    /// Off: this site has no form behind a challenge. A managed-challenge interstitial needs
    /// nothing here either — that is a Cloudflare-served document carrying its own policy.
    #[serde(default)]
    pub turnstile: bool,

    /// Admit the Cloudflare Web Analytics beacon and the endpoint it reports to.
    ///
    /// Off: this site measures nothing, which the privacy page states. Only for the *manual*
    /// snippet — the automatic edge injection is an inline script and needs `script_nonce`
    /// instead.
    #[serde(default)]
    pub web_analytics: bool,
}

impl CloudflareConfig {
    /// The nonce is the default; see the field.
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

/// A `[csp]` block that cannot be served.
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
