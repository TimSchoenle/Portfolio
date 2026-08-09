//! Public JSON API handlers, mirroring the original Next.js portfolio plus
//! Kubernetes probe endpoints. Every response derives solely from compile-time
//! data, so the documents only change on redeploy.

use std::path::Path;
use std::sync::LazyLock;

use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
};
use portfolio_config::AssetsConfig;
use portfolio_data::profile::{Profile, ProfileWithSchema};
use serde::Serialize;
use time::{OffsetDateTime, macros::format_description};

// ── Profile ───────────────────────────────────────────────────────────────

static PROFILE: LazyLock<ProfileWithSchema> = LazyLock::new(portfolio_data::profile::profile);
static SCHEMA: LazyLock<schemars::Schema> = LazyLock::new(|| schemars::schema_for!(Profile));

/// The profile document and its schema derive solely from compile-time data, so
/// they only change on redeploy: a one-hour public TTL keeps them edge/browser
/// cacheable while staying fresh enough.
const CACHE_CONTROL: &str = "public, max-age=3600";

/// `GET /api/v1/profile` — the static, language-neutral profile document.
pub async fn profile() -> impl IntoResponse {
    ([(header::CACHE_CONTROL, CACHE_CONTROL)], Json(&*PROFILE))
}

/// `GET /api/v1/profile/schema` — JSON Schema describing the profile document.
pub async fn schema() -> impl IntoResponse {
    ([(header::CACHE_CONTROL, CACHE_CONTROL)], Json(&*SCHEMA))
}

// ── Health / probes ─────────────────────────────────────────────────────────

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    timestamp: String,
}

/// ISO 8601 / RFC 3339 timestamp with millisecond precision and a `Z` suffix,
/// matching the original endpoint's `new Date().toISOString()`.
fn now_iso8601() -> String {
    let format =
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z");
    OffsetDateTime::now_utc()
        .format(format)
        .expect("static format description is always valid")
}

/// `GET /api/health` — reports service status and the current UTC time.
pub async fn health() -> impl IntoResponse {
    (
        [(header::CACHE_CONTROL, "no-store")],
        Json(HealthResponse {
            status: "healthy",
            timestamp: now_iso8601(),
        }),
    )
}

/// Liveness probe. Succeeds while the async runtime can service the request; the
/// server holds no degradable state, so this is a cheap "is the process wedged?"
/// check.
pub async fn live() -> impl IntoResponse {
    (
        [(header::CACHE_CONTROL, "no-store")],
        Json(HealthResponse {
            status: "alive",
            timestamp: now_iso8601(),
        }),
    )
}

/// Whether the directory at `dist` contains a servable SPA/SSR entrypoint.
fn assets_ready(dist: &Path) -> bool {
    dist.join("index.html").is_file()
}

/// Readiness probe. Confirms the built client assets are present under the
/// configured bundle directory before Kubernetes routes traffic to this
/// instance.
pub async fn ready(State(assets): State<AssetsConfig>) -> impl IntoResponse {
    let (status, payload) = if assets_ready(assets.dist_dir()) {
        (StatusCode::OK, "ready")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "unavailable")
    };

    (
        status,
        [(header::CACHE_CONTROL, "no-store")],
        Json(HealthResponse {
            status: payload,
            timestamp: now_iso8601(),
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use portfolio_data::CONFIG;
    use serde_json::Value;

    async fn json_body(response: axum::response::Response) -> Value {
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn profile_endpoint_serves_the_document() {
        let doc = json_body(profile().await.into_response()).await;
        assert_eq!(doc["email"], CONFIG.email);
        assert_eq!(
            doc["$schema"],
            format!("{}{}", CONFIG.url, portfolio_data::profile::SCHEMA_PATH)
        );
        assert!(!doc["skills"]["languages"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn schema_endpoint_describes_the_profile() {
        let schema = json_body(schema().await.into_response()).await;
        assert_eq!(schema["type"], "object");
        for field in ["email", "skills", "socials", "title", "website"] {
            assert!(
                schema["properties"].get(field).is_some(),
                "schema missing property {field}"
            );
        }
        assert_eq!(schema["properties"]["email"]["format"], "email");
        assert_eq!(schema["properties"]["website"]["format"], "uri");
    }

    #[tokio::test]
    async fn reports_healthy_and_disables_caching() {
        let response = health().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");

        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["status"], "healthy");
        let timestamp = body["timestamp"].as_str().unwrap();
        assert_eq!(timestamp.len(), 24, "unexpected timestamp: {timestamp}");
        assert!(timestamp.ends_with('Z'), "timestamp not UTC: {timestamp}");
        assert_eq!(&timestamp[10..11], "T", "missing date/time separator");
    }

    #[test]
    fn readiness_requires_the_entrypoint() {
        let dir = std::env::temp_dir().join(format!("portfolio-ready-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        assert!(!assets_ready(&dir));
        std::fs::write(dir.join("index.html"), b"<!doctype html>").unwrap();
        assert!(assets_ready(&dir));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
