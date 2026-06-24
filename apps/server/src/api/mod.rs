//! HTTP API, mirroring the endpoints of the original Next.js portfolio.
//!
//! - `GET /api/health` — liveness probe with the current UTC time.
//! - `GET /api/v1/profile` — static, language-neutral profile document.
//! - `GET /api/v1/profile/schema` — JSON Schema for the profile document.

mod health;
mod profile;

use axum::{Router, routing::get};

/// The API sub-router. Mounted by the server ahead of the SPA fallback.
pub fn router() -> Router {
    Router::new()
        .route("/api/health", get(health::health))
        .route("/api/v1/profile", get(profile::profile))
        .route("/api/v1/profile/schema", get(profile::schema))
}
