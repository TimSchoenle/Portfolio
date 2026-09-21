//! The legal documents on the server: the catalog built from `legal.*` at boot, and everything
//! that reads it.
//!
//! Four readers share one [`Legal`] handle:
//!
//! - the two server functions in [`crate::legal`], through [`indexes`] and [`documents`];
//! - the public JSON routes under [`API_PREFIX`], which `terrace-legal-axum` answers with a
//!   strong `ETag`, `304` and `Vary: Accept-Language`;
//! - the sitemap and the render cache's allowlist, through [`hosted_paths`];
//! - the permanent redirects from the paths the documents had before they were configuration.
//!
//! # Why the server functions reach a process global
//!
//! A Dioxus server function has no router state to extract, and `dioxus::serve` owns the router
//! it would be attached to. The configuration is read once, before the runtime exists, and never
//! rebuilt (see `portfolio_config` for why the workspace takes no reload supervisor), so a
//! [`OnceLock`] set in [`install`] holds exactly what a state parameter would have: one value
//! for the life of the process. The JSON routes and the sitemap take the handle as ordinary
//! router state instead, which keeps them testable without the global.

use std::sync::OnceLock;

use axum::Router;
use axum::response::Redirect;
use axum::routing::get;
use portfolio_config::{REQUIRED_DOCUMENTS, legal_catalog_builder};
use portfolio_data::LANGUAGES;
use terrace_legal::{ConfigIssues, Legal, LegalConfig, Negotiation};

use crate::legal::{LegalDocuments, LegalIndexes, document_path, is_hosted, per_language};

/// Where the JSON routes are mounted: the index at the prefix itself, a document below it.
pub(super) const API_PREFIX: &str = "/api/v1/legal";

/// The handle the server functions read. Set once, by [`install`].
static LEGAL: OnceLock<Legal> = OnceLock::new();

/// Validates `config` into the served handle.
///
/// The rules are this deployment's (`portfolio_config::legal_catalog_builder`) on top of
/// `terrace-legal`'s own, and every problem is reported at once, under the key the operator
/// wrote (`legal.documents.privacy.body.de`). Warnings — accepted, but worth a look — are
/// printed to stderr: this runs before any logger exists, for the same reason the refusal does.
///
/// # Errors
///
/// Every problem that keeps `config` from being served.
pub(super) fn build(config: &LegalConfig) -> Result<Legal, ConfigIssues> {
    let builder = legal_catalog_builder(&LANGUAGES);
    let catalog = builder
        .build(config)
        .map_err(|issues| issues.with_prefix("legal"))?;
    for warning in catalog.warnings() {
        eprintln!("portfolio: legal.{}: {}", warning.key(), warning.message());
    }
    Ok(Legal::with_builder(catalog, builder))
}

/// Makes `legal` the handle the server functions answer from.
///
/// A second call is ignored rather than replacing the first: `serve` calls this once, before the
/// runtime starts, and a handle swapped under requests already in flight is the reload this
/// workspace deliberately does not have.
pub(super) fn install(legal: Legal) {
    let _ = LEGAL.set(legal);
}

/// The published documents' index for every site language, or `None` before [`install`].
pub(crate) fn indexes() -> Option<LegalIndexes> {
    let legal = LEGAL.get()?;
    per_language(|lang| Some(legal.index(&negotiation(lang)).value))
}

/// The hosted document `slug` in every site language, or `None` when nothing hosted is published
/// under it — or before [`install`].
///
/// `portfolio_config`'s rules guarantee a text per site language, so a document is either
/// present in all of them or absent; the negotiation never has to fall back across languages.
pub(crate) fn documents(slug: &str) -> Option<LegalDocuments> {
    let legal = LEGAL.get()?;
    per_language(|lang| {
        legal
            .document(slug, &negotiation(lang))
            .ok()
            .map(|served| (*served.value).clone())
    })
}

/// The route path of every hosted document, in catalog order.
///
/// External documents are absent: they have no page on this site to list or to cache.
pub(super) fn hosted_paths(legal: &Legal) -> Vec<String> {
    legal
        .index(&Negotiation::default())
        .value
        .iter()
        .filter(|entry| is_hosted(entry))
        .map(|entry| document_path(&entry.slug))
        .collect()
}

/// The JSON routes and the redirects from the pre-configuration paths.
///
/// Every required document had a top-level route before the documents were configuration
/// (`/imprint`, `/privacy`), and those URLs are printed on other sites and cached by search
/// engines. A permanent redirect keeps them working and tells a crawler to move its entry,
/// rather than turning them into the 404 page.
pub(super) fn router(legal: Legal) -> Router {
    let redirects = REQUIRED_DOCUMENTS
        .into_iter()
        .fold(Router::new(), |router, slug| {
            let target = document_path(slug);
            router.route(
                &format!("/{slug}"),
                get(move || std::future::ready(Redirect::permanent(&target))),
            )
        });
    redirects.nest(
        API_PREFIX,
        terrace_legal_axum::router::<Legal>().with_state(legal),
    )
}

/// What a site language asks for: that language, with no `Accept-Language` to consult, because
/// the site negotiated the language already (`crate::i18n`).
fn negotiation(lang: &str) -> Negotiation {
    Negotiation::from_request(Some(lang), None)
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode, header};
    use terrace_legal::LegalDocument;
    use tower::ServiceExt;

    /// A servable configuration: both required documents in both languages, and one external
    /// document, which has a link but no page.
    pub(crate) fn fixture() -> Legal {
        let mut config = LegalConfig {
            default_locale: Some("en".to_owned()),
            ..LegalConfig::default()
        };
        for (order, slug) in (1..).zip(REQUIRED_DOCUMENTS) {
            let mut document = LegalDocument {
                order,
                ..LegalDocument::default()
            };
            for lang in LANGUAGES {
                document
                    .body
                    .insert(lang.to_owned(), format!("## {slug}\n\nText in {lang}."));
                document
                    .title
                    .insert(lang.to_owned(), format!("{slug} ({lang})"));
            }
            config.documents.insert(slug.to_owned(), document);
        }
        let external = LegalDocument {
            url: Some("https://example.org/terms".to_owned()),
            order: 99,
            ..LegalDocument::default()
        };
        config.documents.insert("terms".to_owned(), external);
        build(&config).expect("the fixture is a servable configuration")
    }

    async fn get(router: Router, uri: &str) -> axum::response::Response {
        router
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[test]
    fn only_hosted_documents_have_a_path() {
        assert_eq!(
            hosted_paths(&fixture()),
            ["/legal/imprint", "/legal/privacy"]
        );
    }

    #[test]
    fn a_configuration_missing_a_required_document_is_refused_under_its_full_key() {
        let issues = build(&LegalConfig::default()).expect_err("nothing is published");
        let keys: Vec<String> = issues.iter().map(terrace_legal::ConfigIssue::key).collect();
        assert!(
            keys.contains(&"legal.documents.imprint".to_owned()),
            "{keys:?}"
        );
    }

    #[tokio::test]
    async fn the_old_top_level_paths_redirect_permanently() {
        for slug in REQUIRED_DOCUMENTS {
            let response = get(router(fixture()), &format!("/{slug}")).await;
            assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
            assert_eq!(
                response.headers()[header::LOCATION],
                format!("/legal/{slug}").as_str()
            );
        }
    }

    #[tokio::test]
    async fn the_json_index_lists_every_document_in_order() {
        let response = get(router(fixture()), "/api/v1/legal?lang=de").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().contains_key(header::ETAG));
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let index: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let slugs: Vec<&str> = index
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["slug"].as_str().unwrap())
            .collect();
        assert_eq!(slugs, ["imprint", "privacy", "terms"]);
        assert_eq!(index[0]["title"], "imprint (de)");
    }

    /// The conditional half of the contract, which is the reason to serve through
    /// `terrace-legal-axum` rather than a handler of our own.
    #[tokio::test]
    async fn a_document_answers_a_matching_etag_with_not_modified() {
        let first = get(router(fixture()), "/api/v1/legal/privacy?lang=en").await;
        assert_eq!(first.status(), StatusCode::OK);
        let etag = first.headers()[header::ETAG].clone();

        let second = router(fixture())
            .oneshot(
                Request::builder()
                    .uri("/api/v1/legal/privacy?lang=en")
                    .header(header::IF_NONE_MATCH, etag)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(second.status(), StatusCode::NOT_MODIFIED);
    }

    #[tokio::test]
    async fn an_unknown_or_external_document_is_not_served() {
        for slug in ["nope", "terms", "..%2F..%2Fetc%2Fpasswd"] {
            let response = get(router(fixture()), &format!("/api/v1/legal/{slug}")).await;
            assert!(
                response.status().is_client_error(),
                "{slug}: {}",
                response.status()
            );
        }
    }
}
