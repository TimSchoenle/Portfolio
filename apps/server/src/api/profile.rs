//! `GET /api/v1/profile` and `GET /api/v1/profile/schema`.
//!
//! Both responses derive solely from compile-time data, so they are built
//! once and cached for the process lifetime, mirroring the `force-static`
//! behaviour of the original Next.js routes. The document and its schema share
//! the [`portfolio_data::profile`] model, so the served data and the published
//! schema cannot drift apart.

use std::sync::LazyLock;

use axum::{Json, response::IntoResponse};
use portfolio_data::profile::{Profile, ProfileWithSchema};

static PROFILE: LazyLock<ProfileWithSchema> = LazyLock::new(portfolio_data::profile::profile);
static SCHEMA: LazyLock<schemars::Schema> = LazyLock::new(|| schemars::schema_for!(Profile));

pub async fn profile() -> impl IntoResponse {
    Json(&*PROFILE)
}

pub async fn schema() -> impl IntoResponse {
    Json(&*SCHEMA)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::to_bytes,
        http::{StatusCode, header},
    };
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
}
