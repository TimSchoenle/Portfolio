//! `robots.txt`, `sitemap.xml` and `site.webmanifest`, built from [`CONFIG`] — and, for the
//! sitemap, from the legal documents the configuration hosts.
//!
//! Three whole documents a crawler fetches by a fixed path, which is what separates them from
//! the rest of the site's metadata. Anything a crawler reads *inside* a page — the title, the
//! canonical link, the Open Graph tags — is a `document::` element in the component that owns
//! the route, so it is rendered with the page rather than assembled beside it.

use std::sync::LazyLock;

use std::fmt::Write as _;

use axum::{body::Bytes, http::header, response::IntoResponse};
use portfolio_data::{APP_ICONS, CONFIG, DEFAULT_LANGUAGE, SITE_LANGUAGES};
use serde_json::json;

use crate::i18n::localized_url;

/// These two documents are built from [`CONFIG`] alone, so each is rendered once
/// on first request rather than rebuilt on every one. The sitemap also lists
/// configured pages, so the server renders it once at startup instead and hands
/// it to [`sitemap`].
///
/// The builders below stay separate functions so the tests exercise the
/// construction itself rather than whatever a `LazyLock` happens to be holding.
static ROBOTS_TXT: LazyLock<String> = LazyLock::new(robots_txt);
static WEBMANIFEST_JSON: LazyLock<String> = LazyLock::new(webmanifest_json);

/// `GET /robots.txt`.
pub async fn robots() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        ROBOTS_TXT.as_str(),
    )
}

/// `GET /sitemap.xml`, answering with `xml` as [`sitemap_xml`] rendered it at startup.
///
/// [`Bytes`] so each response shares the rendered document instead of copying it.
pub fn sitemap(xml: Bytes) -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        xml,
    )
}

/// `GET /site.webmanifest`.
pub async fn webmanifest() -> impl IntoResponse {
    (
        [(
            header::CONTENT_TYPE,
            "application/manifest+json; charset=utf-8",
        )],
        WEBMANIFEST_JSON.as_str(),
    )
}

fn robots_txt() -> String {
    format!(
        "User-Agent: *\nAllow: /\nDisallow: /api/\n\nSitemap: {}/sitemap.xml\n",
        CONFIG.url
    )
}

/// The sitemap: the fixed routes of `crate::routes::Route`, with `legal_paths` — each a hosted
/// legal document's route, in the operator's order — between the home page and the licenses.
///
/// Every page is listed once per site language, at the address [`localized_url`] gives it, and
/// each entry names all its language variants (and `x-default`) as `hreflang` alternates — the
/// sitemap form of the `<link rel="alternate">` tags the page itself carries.
pub fn sitemap_xml(legal_paths: &[String]) -> String {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\" \
         xmlns:xhtml=\"http://www.w3.org/1999/xhtml\">\n",
    );
    let mut page = |path: &str, changefreq: &str, priority: &str| {
        let mut alternates = String::new();
        for language in &SITE_LANGUAGES {
            alternates.push_str(&alternate(language.code, &localized_url(path, language)));
        }
        alternates.push_str(&alternate(
            "x-default",
            &localized_url(path, DEFAULT_LANGUAGE),
        ));
        for language in &SITE_LANGUAGES {
            let _ = write!(
                xml,
                "  <url>\n    <loc>{}</loc>\n{alternates}    <changefreq>{changefreq}</changefreq>\n    <priority>{priority}</priority>\n  </url>\n",
                xml_escape(&localized_url(path, language))
            );
        }
    };
    page("/", "weekly", "1.0");
    for path in legal_paths {
        page(path, "monthly", "0.5");
    }
    // Regenerated from the dependency set on every build, so it changes as
    // often as the site is deployed rather than as rarely as a legal text.
    page("/licenses", "weekly", "0.3");
    xml.push_str("</urlset>\n");
    xml
}

/// One `<xhtml:link rel="alternate">` line.
fn alternate(hreflang: &str, href: &str) -> String {
    format!(
        "    <xhtml:link rel=\"alternate\" hreflang=\"{hreflang}\" href=\"{}\"/>\n",
        xml_escape(href)
    )
}

/// Escapes the characters XML reserves in text and attribute values.
fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The favicon, then the raster icons an install prompt requires.
fn manifest_icons() -> serde_json::Value {
    let mut icons = vec![json!({
        "src": "/favicon.svg",
        "sizes": "any",
        "type": "image/svg+xml",
        "purpose": "any",
    })];
    for (size, name) in APP_ICONS {
        icons.push(json!({
            "src": format!("/{name}"),
            "sizes": format!("{size}x{size}"),
            "type": "image/png",
            "purpose": "any",
        }));
    }
    serde_json::Value::Array(icons)
}

fn webmanifest_json() -> String {
    let c = &CONFIG;
    let manifest = json!({
        "name": c.title,
        "short_name": c.full_name,
        "description": c.description,
        "id": "portfolio",
        "start_url": "/",
        "display": "standalone",
        "background_color": "#0a0d14",
        "theme_color": "#0a0d14",
        "categories": ["productivity", "portfolio", "developer"],
        "icons": manifest_icons(),
    });
    serde_json::to_string_pretty(&manifest).expect("manifest serializes") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn robots_allows_crawling_but_blocks_the_api_and_links_the_sitemap() {
        let robots = robots_txt();
        assert!(robots.contains("User-Agent: *"));
        assert!(robots.contains("Allow: /"));
        assert!(robots.contains("Disallow: /api/"));
        assert!(robots.contains(&format!("Sitemap: {}/sitemap.xml", CONFIG.url)));
    }

    #[test]
    fn sitemap_lists_every_public_route() {
        let legal = ["/legal/imprint".to_owned(), "/legal/privacy".to_owned()];
        let xml = sitemap_xml(&legal);
        assert!(xml.starts_with("<?xml"));
        for path in ["/legal/imprint", "/legal/privacy", "/licenses"] {
            assert!(
                xml.contains(&format!("<loc>{}{path}</loc>", CONFIG.url)),
                "sitemap missing route {path}"
            );
            assert!(
                xml.contains(&format!("<loc>{}{path}?lang=de</loc>", CONFIG.url)),
                "sitemap missing the German {path}"
            );
        }
        assert!(xml.contains(&format!("<loc>{}</loc>", CONFIG.url)));
        // Four pages, each once per language, and every entry names the default variant.
        assert_eq!(xml.matches("<url>").count(), 4 * SITE_LANGUAGES.len());
        assert_eq!(
            xml.matches("hreflang=\"x-default\"").count(),
            4 * SITE_LANGUAGES.len()
        );
    }

    #[test]
    fn webmanifest_is_valid_json_derived_from_config() {
        let manifest: serde_json::Value =
            serde_json::from_str(&webmanifest_json()).expect("manifest is valid JSON");
        assert_eq!(manifest["name"], CONFIG.title);
        assert_eq!(manifest["short_name"], CONFIG.full_name);
        assert_eq!(manifest["start_url"], "/");
        let icons = manifest["icons"].as_array().expect("an icon list");
        for (size, _) in APP_ICONS {
            assert!(
                icons
                    .iter()
                    .any(|icon| icon["sizes"] == format!("{size}x{size}")),
                "no {size}px icon"
            );
        }
    }
}
