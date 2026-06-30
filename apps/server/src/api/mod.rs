//! HTTP API, mirroring the endpoints of the original Next.js portfolio plus
//! dedicated Kubernetes probe endpoints.
//!
//! - `GET /api/health` — general health report with the current UTC time.
//! - `GET /api/health/live`  (alias `GET /livez`)  — liveness probe.
//! - `GET /api/health/ready` (alias `GET /readyz`) — readiness probe.
//! - `GET /api/v1/profile` — static, language-neutral profile document.
//! - `GET /api/v1/profile/schema` — JSON Schema for the profile document.

mod health;
mod profile;

use axum::{Router, routing::get};

/// The API sub-router. Mounted by the server ahead of the SPA fallback.
pub fn router() -> Router {
    Router::new()
        .route("/api/health", get(health::health))
        .route("/api/health/live", get(health::live))
        .route("/api/health/ready", get(health::ready))
        // Short, kubelet-friendly aliases so probes can target the
        // conventional `/livez` and `/readyz` paths directly.
        .route("/livez", get(health::live))
        .route("/readyz", get(health::ready))
        .route("/api/v1/profile", get(profile::profile))
        .route("/api/v1/profile/schema", get(profile::schema))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use portfolio_data::CONFIG;
    use serde_json::Value;
    use tower::ServiceExt;

    /// Dispatches a `GET` through the real API router and returns the response.
    async fn get(path: &str) -> axum::response::Response {
        router()
            .oneshot(
                Request::builder()
                    .uri(path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap()
    }

    async fn json(response: axum::response::Response) -> Value {
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn health_route_is_wired_and_uncached() {
        let response = get("/api/health").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        assert_eq!(json(response).await["status"], "healthy");
    }

    #[tokio::test]
    async fn liveness_routes_are_wired_and_uncached() {
        for path in ["/api/health/live", "/livez"] {
            let response = get(path).await;
            assert_eq!(response.status(), StatusCode::OK, "{path}");
            assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
            assert_eq!(json(response).await["status"], "alive");
        }
    }

    #[tokio::test]
    async fn readiness_routes_are_wired_and_uncached() {
        // The readiness verdict depends on DIST_DIR (asserted in unit tests in
        // the `health` module); here we only assert the routes are wired,
        // never cached, and report a well-formed status either way.
        for path in ["/api/health/ready", "/readyz"] {
            let response = get(path).await;
            assert!(
                matches!(
                    response.status(),
                    StatusCode::OK | StatusCode::SERVICE_UNAVAILABLE
                ),
                "{path}: unexpected status {}",
                response.status()
            );
            assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
            let status = json(response).await["status"].as_str().unwrap().to_owned();
            assert!(status == "ready" || status == "unavailable", "{path}: {status}");
        }
    }

    #[tokio::test]
    async fn profile_route_serves_the_cached_document() {
        let response = get("/api/v1/profile").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "public, max-age=3600"
        );
        let doc = json(response).await;
        assert_eq!(doc["email"], CONFIG.email);
        assert_eq!(
            doc["$schema"],
            format!("{}{}", CONFIG.url, portfolio_data::profile::SCHEMA_PATH)
        );
    }

    #[tokio::test]
    async fn schema_route_describes_the_profile() {
        let response = get("/api/v1/profile/schema").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(json(response).await["type"], "object");
    }

    #[tokio::test]
    async fn unknown_api_path_is_not_found() {
        let response = get("/api/does-not-exist").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn profile_rejects_non_get_methods() {
        let response = router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/profile")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }
}
