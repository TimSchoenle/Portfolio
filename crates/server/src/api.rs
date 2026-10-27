//! The handlers of the public JSON API and of the health probes.
//!
//! The two profile documents are functions of compile-time data, so each is rendered once and
//! changes only on a redeploy. The probes carry a live timestamp and are built per request. The
//! router in `apps/web` decides the paths; each handler's documentation names the one it is
//! mounted at.
//!
//! The profile endpoint's shape is a contract with consumers older than this crate; see
//! [`portfolio_data::profile`].

#![expect(
    clippy::unused_async,
    reason = "axum takes a handler as an `async fn` even when its body never awaits"
)]

use std::path::Path;
use std::sync::LazyLock;

use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
    response::IntoResponse,
};
use portfolio_config::AssetsConfig;
use portfolio_data::profile::Profile;
use serde::Serialize;
use time::{OffsetDateTime, macros::format_description};

// ── Profile ───────────────────────────────────────────────────────────────

/// The profile document and its JSON Schema, rendered to JSON once.
///
/// Both derive solely from compile-time data, so every caller receives the same
/// bytes for the whole life of the process. Serializing once here reduces the
/// handlers to a pair of headers and a `&'static str`, where `Json` would
/// re-serialize the whole document per request.
static PROFILE_JSON: LazyLock<String> = LazyLock::new(|| {
    serde_json::to_string(&portfolio_data::profile::profile())
        .expect("the profile document serializes")
});
static SCHEMA_JSON: LazyLock<String> = LazyLock::new(|| {
    serde_json::to_string(&schemars::schema_for!(Profile)).expect("the profile schema serializes")
});

/// The `Cache-Control` of both profile documents: public, for one hour.
const CACHE_CONTROL: &str = "public, max-age=3600";

/// The content type of the two profile documents, exactly the value `Json` sends.
// Stated because the profile handlers bypass `Json`, which would otherwise set it.
const JSON_CONTENT_TYPE: &str = "application/json";

/// Serves the language-neutral profile document, mounted at `GET /api/v1/profile`.
pub async fn profile() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, JSON_CONTENT_TYPE),
            (header::CACHE_CONTROL, CACHE_CONTROL),
        ],
        PROFILE_JSON.as_str(),
    )
}

/// Serves the JSON Schema of the profile document, mounted at `GET /api/v1/profile/schema`.
pub async fn schema() -> impl IntoResponse {
    (
        [
            (header::CONTENT_TYPE, JSON_CONTENT_TYPE),
            (header::CACHE_CONTROL, CACHE_CONTROL),
        ],
        SCHEMA_JSON.as_str(),
    )
}

// ── Health / probes ─────────────────────────────────────────────────────────

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    timestamp: String,
}

/// Returns the current UTC time as `YYYY-MM-DDTHH:MM:SS.sssZ`.
///
/// Millisecond precision and a literal `Z`, which is what `Date.prototype.toISOString` emits and
/// therefore what anything already parsing this endpoint expects.
fn now_iso8601() -> String {
    let format =
        format_description!("[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z");
    OffsetDateTime::now_utc()
        .format(format)
        .expect("static format description is always valid")
}

/// Reports the service status and the current UTC time, mounted at `GET /api/health`.
pub async fn health() -> impl IntoResponse {
    (
        [(header::CACHE_CONTROL, "no-store")],
        Json(HealthResponse {
            status: "healthy",
            timestamp: now_iso8601(),
        }),
    )
}

/// Answers the liveness probe, succeeding while the async runtime can service a request.
///
/// The server holds no state that can degrade, so this only tells whether the process is
/// wedged.
pub async fn live() -> impl IntoResponse {
    (
        [(header::CACHE_CONTROL, "no-store")],
        Json(HealthResponse {
            status: "alive",
            timestamp: now_iso8601(),
        }),
    )
}

/// Reports whether the directory at `dist` contains the client bundle's `index.html`.
fn assets_ready(dist: &Path) -> bool {
    dist.join("index.html").is_file()
}

/// Answers the readiness probe: `200` when the client bundle is present in the configured
/// directory, `503` otherwise.
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
        assert_ne!(doc["skills"]["languages"].as_array().unwrap().len(), 0);
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
