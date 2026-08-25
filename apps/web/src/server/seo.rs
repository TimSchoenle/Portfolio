//! `robots.txt`, `sitemap.xml` and `site.webmanifest`, built from [`CONFIG`].
//!
//! Three whole documents a crawler fetches by a fixed path, which is what separates them from
//! the rest of the site's metadata. Anything a crawler reads *inside* a page — the title, the
//! canonical link, the Open Graph tags — is a `document::` element in the component that owns
//! the route, so it is rendered with the page rather than assembled beside it.

use std::sync::LazyLock;

use axum::{http::header, response::IntoResponse};
use portfolio_data::CONFIG;
use serde_json::json;

/// All three documents are built from [`CONFIG`] alone, so each is rendered once
/// on first request rather than rebuilt on every one — the manifest in
/// particular ran a full pretty-printing serialization each time it was fetched.
///
/// The builders below stay separate functions so the tests exercise the
/// construction itself rather than whatever a `LazyLock` happens to be holding.
static ROBOTS_TXT: LazyLock<String> = LazyLock::new(robots_txt);
static SITEMAP_XML: LazyLock<String> = LazyLock::new(sitemap_xml);
static WEBMANIFEST_JSON: LazyLock<String> = LazyLock::new(webmanifest_json);

/// `GET /robots.txt`.
pub async fn robots() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        ROBOTS_TXT.as_str(),
    )
}

/// `GET /sitemap.xml`.
pub async fn sitemap() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        SITEMAP_XML.as_str(),
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

fn sitemap_xml() -> String {
    // Paths mirror the routes in `app::routes::Route`.
    let entry = |path: &str, changefreq: &str, priority: &str| {
        format!(
            "  <url>\n    <loc>{}{path}</loc>\n    <changefreq>{changefreq}</changefreq>\n    <priority>{priority}</priority>\n  </url>\n",
            CONFIG.url
        )
    };
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n\
         {}{}{}{}</urlset>\n",
        entry("/", "weekly", "1.0"),
        entry("/imprint", "monthly", "0.5"),
        entry("/privacy", "monthly", "0.5"),
        // Regenerated from the dependency set on every build, so it changes as
        // often as the site is deployed rather than as rarely as a legal text.
        entry("/licenses", "weekly", "0.3"),
    )
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
        "icons": [{
            "src": "/favicon.svg",
            "sizes": "any",
            "type": "image/svg+xml",
            "purpose": "any",
        }],
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
        let xml = sitemap_xml();
        assert!(xml.starts_with("<?xml"));
        for path in ["/", "/imprint", "/privacy", "/licenses"] {
            assert!(
                xml.contains(&format!("<loc>{}{path}</loc>", CONFIG.url)),
                "sitemap missing route {path}"
            );
        }
        assert_eq!(xml.matches("<url>").count(), 4);
    }

    #[test]
    fn webmanifest_is_valid_json_derived_from_config() {
        let manifest: serde_json::Value =
            serde_json::from_str(&webmanifest_json()).expect("manifest is valid JSON");
        assert_eq!(manifest["name"], CONFIG.title);
        assert_eq!(manifest["short_name"], CONFIG.full_name);
        assert_eq!(manifest["start_url"], "/");
        assert!(manifest["icons"].as_array().is_some_and(|i| !i.is_empty()));
    }
}
