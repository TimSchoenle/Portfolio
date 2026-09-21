//! What this deployment requires of the legal documents an operator configures.
//!
//! The documents themselves — their text, titles, dates and order — are
//! [`terrace_legal::LegalConfig`], nested under `legal` in [`ServerConfig`](crate::ServerConfig).
//! `terrace-legal` decides whether a configuration can be *served* at all. This module adds what
//! this site cannot run without, as two [`Rule`]s evaluated in the same pass, so an operator sees
//! every problem in one refused boot rather than one per restart:
//!
//! - [`REQUIRED_DOCUMENTS`] are published. A German site owes its readers an imprint (§ 5 DDG)
//!   and a privacy notice (Art. 13 GDPR), and a deployment that forgets either must not start.
//! - Every document served from here has a text in every language the site renders. The GDPR
//!   wants the notice in the reader's language, and the site offers exactly the languages its
//!   interface is translated into; a missing German privacy text would otherwise fall back to
//!   English silently, on a page whose chrome is German.

use terrace_legal::model::LocaleTag;
use terrace_legal::{CatalogBuilder, ConfigIssue, LegalDocument, RequiredDocuments, Rule};

/// The slugs a deployment has to publish.
///
/// They are also the paths the documents are served under (`/legal/imprint`), which is why the
/// old top-level `/imprint` and `/privacy` routes can redirect to them unconditionally.
pub const REQUIRED_DOCUMENTS: [&str; 2] = ["imprint", "privacy"];

/// The validation this deployment applies to `legal.*`, for the catalog and every later rebuild.
///
/// `locales` are the languages the site renders, from `portfolio_data::LANGUAGES`. They are a
/// parameter rather than a copy here because this crate does not depend on the data crate, and a
/// second list would be a list that can disagree with the one the interface is built from.
#[must_use]
pub fn legal_catalog_builder(locales: &[&'static str]) -> CatalogBuilder {
    CatalogBuilder::new()
        .rule(RequiredDocuments::new(REQUIRED_DOCUMENTS))
        .rule(EveryLocale {
            locales: locales.to_vec(),
        })
}

/// Refuses a hosted document that lacks a text in one of the site's languages.
///
/// A body keyed by a regional variant counts for its language: `de-AT` satisfies `de`, because
/// that is also what the negotiation serves a reader who asked for `de`. External documents are
/// exempt — they are a link, and what language the page behind it is in is not this site's to
/// check.
struct EveryLocale {
    locales: Vec<&'static str>,
}

impl Rule for EveryLocale {
    fn check_document(&self, _slug: &str, document: &LegalDocument) -> Vec<ConfigIssue> {
        if document.url.is_some() {
            return Vec::new();
        }
        // A key that is not a locale at all is reported by the built-in checks; skipping it here
        // keeps the report to one issue per mistake.
        let published: Vec<LocaleTag> = document
            .body
            .keys()
            .filter_map(|key| key.parse().ok())
            .collect();
        self.locales
            .iter()
            .filter(|locale| {
                !published
                    .iter()
                    .any(|tag| tag.language().eq_ignore_ascii_case(locale))
            })
            .map(|locale| {
                ConfigIssue::new(
                    ["body", *locale],
                    "is missing: every hosted document needs a text in every language the site \
                     renders",
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{REQUIRED_DOCUMENTS, legal_catalog_builder};
    use terrace_legal::{LegalConfig, LegalDocument};

    const LOCALES: [&str; 2] = ["en", "de"];

    fn hosted(locales: &[&str]) -> LegalDocument {
        let mut document = LegalDocument::default();
        for locale in locales {
            document
                .body
                .insert((*locale).to_owned(), format!("## Text\n\nIn {locale}."));
        }
        document
    }

    fn complete() -> LegalConfig {
        let mut config = LegalConfig::default();
        config.default_locale = Some("en".to_owned());
        for slug in REQUIRED_DOCUMENTS {
            config.documents.insert(slug.to_owned(), hosted(&LOCALES));
        }
        config
    }

    fn keys(config: &LegalConfig) -> Vec<String> {
        legal_catalog_builder(&LOCALES)
            .build(config)
            .expect_err("the configuration must be refused")
            .with_prefix("legal")
            .iter()
            .map(terrace_legal::ConfigIssue::key)
            .collect()
    }

    #[test]
    fn a_complete_configuration_builds() {
        let catalog = legal_catalog_builder(&LOCALES)
            .build(&complete())
            .expect("both documents in both languages is a servable configuration");
        assert_eq!(catalog.len(), REQUIRED_DOCUMENTS.len());
    }

    /// The empty configuration is what a deployment gets when nobody mounted the legal texts, and
    /// it has to name both documents rather than start without them.
    #[test]
    fn a_deployment_without_the_required_documents_is_refused_by_name() {
        let keys = keys(&LegalConfig::default());
        assert!(
            keys.contains(&"legal.documents.imprint".to_owned()),
            "{keys:?}"
        );
        assert!(
            keys.contains(&"legal.documents.privacy".to_owned()),
            "{keys:?}"
        );
    }

    #[test]
    fn a_hosted_document_missing_a_site_language_is_refused() {
        let mut config = complete();
        config
            .documents
            .insert("privacy".to_owned(), hosted(&["en"]));
        assert_eq!(keys(&config), ["legal.documents.privacy.body.de"]);
    }

    /// `de-AT` is German for the negotiation, so it is German for this rule too.
    #[test]
    fn a_regional_variant_satisfies_its_language() {
        let mut config = complete();
        config
            .documents
            .insert("privacy".to_owned(), hosted(&["en", "de-AT"]));
        assert!(legal_catalog_builder(&LOCALES).build(&config).is_ok());
    }

    /// The texts committed under `legal/` are what development, the container smoke test and
    /// the Helm values are all taken from, so they are loaded here through the real dialect and
    /// judged by the real rules. The three fragments are joined into one because each owns a
    /// distinct table, which is also why the loader can read them as a directory.
    #[test]
    fn the_committed_legal_texts_load_and_are_servable() {
        #[derive(serde::Deserialize)]
        struct Root {
            legal: LegalConfig,
        }

        let fragments = [
            include_str!("../../../legal/legal.toml"),
            include_str!("../../../legal/imprint.toml"),
            include_str!("../../../legal/privacy.toml"),
        ]
        .join("\n");

        terrace_config::testing::Harness::over(crate::terrace()).run(|jail| {
            jail.config(&fragments)?;
            let root: Root = jail.load()?;
            let catalog = legal_catalog_builder(&LOCALES)
                .build(&root.legal)
                .unwrap_or_else(|issues| panic!("{issues}"));
            let slugs: Vec<String> = catalog.slugs().map(ToString::to_string).collect();
            assert_eq!(slugs, REQUIRED_DOCUMENTS);
            assert!(catalog.warnings().is_empty(), "{:?}", catalog.warnings());
            Ok(())
        });
    }

    /// An external document is a link; the language of the page behind it is not checkable here.
    #[test]
    fn an_external_document_is_exempt_from_the_language_rule() {
        let mut config = complete();
        let mut external = LegalDocument::default();
        external.url = Some("https://example.org/terms".to_owned());
        config.documents.insert("terms".to_owned(), external);
        assert!(legal_catalog_builder(&LOCALES).build(&config).is_ok());
    }
}
