//! Health and Kubernetes probe endpoints.
//!
//! - `GET /api/health` — general health report with the current UTC time
//!   (kept for backwards compatibility with the original Next.js portfolio).
//! - `GET /api/health/live`  (alias `GET /livez`)  — **liveness**: the process
//!   is running and able to serve requests. A failure tells the kubelet to
//!   restart the container.
//! - `GET /api/health/ready` (alias `GET /readyz`) — **readiness**: the built
//!   SPA is present so static assets and the SPA fallback can be served. A
//!   failure removes the pod from the Service endpoints without restarting it.

use std::path::Path;

use axum::{
    Json,
    http::{StatusCode, header},
    response::IntoResponse,
};
use serde::Serialize;
use time::{OffsetDateTime, macros::format_description};

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

/// Liveness probe. Succeeds for as long as the async runtime can service the
/// request; the server holds no degradable state, so this is intentionally a
/// cheap "is the process wedged?" check.
pub async fn live() -> impl IntoResponse {
    (
        [(header::CACHE_CONTROL, "no-store")],
        Json(HealthResponse {
            status: "alive",
            timestamp: now_iso8601(),
        }),
    )
}

/// Whether the directory at `dist` contains a servable SPA entrypoint. This is
/// the one runtime dependency the server has: without `index.html` neither the
/// static assets nor the SPA fallback can be served.
fn assets_ready(dist: &Path) -> bool {
    dist.join("index.html").is_file()
}

/// Readiness probe. Confirms the built SPA is present under `DIST_DIR` before
/// Kubernetes routes traffic to this instance.
pub async fn ready() -> impl IntoResponse {
    let dist = std::env::var("DIST_DIR").unwrap_or_else(|_| "dist".into());

    let (status, payload) = if assets_ready(Path::new(&dist)) {
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
    use axum::{body::to_bytes, http::StatusCode};
    use serde_json::Value;

    async fn body_json(response: axum::response::Response) -> Value {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn reports_healthy_and_disables_caching() {
        let response = health().await.into_response();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");

        let body = body_json(response).await;

        assert_eq!(body["status"], "healthy");
        // e.g. "2026-06-24T12:34:56.789Z"
        let timestamp = body["timestamp"].as_str().unwrap();
        assert_eq!(timestamp.len(), 24, "unexpected timestamp: {timestamp}");
        assert!(timestamp.ends_with('Z'), "timestamp not UTC: {timestamp}");
        assert_eq!(&timestamp[10..11], "T", "missing date/time separator");
    }

    #[tokio::test]
    async fn liveness_reports_alive_and_is_uncached() {
        let response = live().await.into_response();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(body_json(response).await["status"], "alive");
    }

    #[test]
    fn readiness_requires_the_spa_index() {
        let dir = std::env::temp_dir().join(format!("portfolio-ready-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        // No index.html yet -> not ready.
        assert!(!assets_ready(&dir));

        std::fs::write(dir.join("index.html"), b"<!doctype html>").unwrap();
        assert!(assets_ready(&dir));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
