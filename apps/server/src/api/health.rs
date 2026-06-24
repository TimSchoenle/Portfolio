//! `GET /api/health` — reports service status and the current UTC time.

use axum::{Json, http::header, response::IntoResponse};
use serde::Serialize;
use time::{OffsetDateTime, macros::format_description};

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    timestamp: String,
}

pub async fn health() -> impl IntoResponse {
    // ISO 8601 / RFC 3339 with millisecond precision and a `Z` suffix,
    // matching the original endpoint's `new Date().toISOString()`.
    let format = format_description!(
        "[year]-[month]-[day]T[hour]:[minute]:[second].[subsecond digits:3]Z"
    );
    let timestamp = OffsetDateTime::now_utc()
        .format(format)
        .expect("static format description is always valid");

    (
        [(header::CACHE_CONTROL, "no-store")],
        Json(HealthResponse {
            status: "healthy",
            timestamp,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::to_bytes, http::StatusCode};
    use serde_json::Value;

    #[tokio::test]
    async fn reports_healthy_and_disables_caching() {
        let response = health().await.into_response();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");

        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(body["status"], "healthy");
        // e.g. "2026-06-24T12:34:56.789Z"
        let timestamp = body["timestamp"].as_str().unwrap();
        assert_eq!(timestamp.len(), 24, "unexpected timestamp: {timestamp}");
        assert!(timestamp.ends_with('Z'), "timestamp not UTC: {timestamp}");
        assert_eq!(&timestamp[10..11], "T", "missing date/time separator");
    }
}
