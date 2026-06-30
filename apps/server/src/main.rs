mod api;
mod cache;

use std::net::SocketAddr;

use axum::http::{HeaderValue, header};
use tower_http::{
    compression::CompressionLayer,
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// `'unsafe-inline'` is required for Trunk's WASM bootstrap script,
/// `'wasm-unsafe-eval'` for instantiating the WASM module.
const CSP: &str = "default-src 'self'; \
     script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; \
     style-src 'self' 'unsafe-inline'; \
     font-src 'self'; \
     img-src 'self' data:; \
     connect-src 'self'; \
     object-src 'none'; \
     base-uri 'self'; \
     form-action 'self'; \
     frame-ancestors 'none'; \
     upgrade-insecure-requests";

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let dist = std::env::var("DIST_DIR").unwrap_or_else(|_| "dist".into());

    let spa = ServeDir::new(&dist).fallback(ServeFile::new(format!("{dist}/index.html")));

    let static_header = |name, value: &'static str| {
        SetResponseHeaderLayer::overriding(name, HeaderValue::from_static(value))
    };

    let app = api::router()
        .fallback_service(spa)
        .layer(static_header(header::CONTENT_SECURITY_POLICY, CSP))
        .layer(static_header(header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
        .layer(static_header(header::X_FRAME_OPTIONS, "DENY"))
        .layer(static_header(header::REFERRER_POLICY, "no-referrer"))
        .layer(static_header(
            header::STRICT_TRANSPORT_SECURITY,
            "max-age=31536000; includeSubDomains; preload",
        ))
        .layer(static_header(
            header::HeaderName::from_static("permissions-policy"),
            "camera=(), microphone=(), geolocation=(), interest-cohort=()",
        ))
        // Compress text-y assets (HTML, the wasm-bindgen JS glue, WASM, CSS)
        // with gzip/brotli per the client's Accept-Encoding. Already-compressed
        // payloads (woff2, svg-as-image) are skipped automatically. This is the
        // production substitute for minifying the wasm-bindgen JS, which Trunk
        // cannot currently minify.
        .layer(CompressionLayer::new())
        // Assign a `Cache-Control` TTL per asset class: immutable (1y) for
        // Trunk's content-hashed JS/WASM, a moderate TTL for fonts/resumes,
        // a short one for generated metadata, and revalidate for HTML. Runs
        // last so it only fills in a TTL the handlers/API did not set.
        .layer(axum::middleware::from_fn(cache::set_cache_control))
        .layer(TraceLayer::new_for_http());

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let sigterm = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c  => {}
        _ = sigterm => {}
    }
}
