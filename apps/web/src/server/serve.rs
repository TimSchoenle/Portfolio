//! The process entry point: configuration, the legal catalog and telemetry, then the serve loop.
//!
//! A development build hands the router to `dioxus::serve`, which applies `dx serve`'s hot
//! patches. A release build runs its own loop, which adds the graceful shutdown on `SIGTERM` and
//! `SIGINT` that `dioxus::serve` lacks.

use std::sync::Arc;

#[cfg(not(debug_assertions))]
use axum::Router;

use super::isr;
use super::{config, legal, router, telemetry};

/// Runs the SSR server on `IP`/`PORT`, default `127.0.0.1:8080`, until it is stopped.
///
/// A release build returns once `SIGTERM` or `SIGINT` has drained it. A development build runs
/// under `dioxus::serve` and never returns. Configuration is read once, before the runtime exists,
/// and a configuration that cannot be loaded ends the process with a non-zero exit code.
// Failing at start-up means the container never reports ready instead of serving a degraded site.
pub fn serve() {
    let config = Arc::new(config::load());
    // The legal catalog is part of the configuration's validity: a deployment without an
    // imprint and a privacy notice in every language is refused here, with every missing key
    // named, before anything binds a port.
    let legal = match legal::build(&config.legal) {
        Ok(legal) => legal,
        Err(issues) => config::refuse(&issues),
    };
    legal::install(legal.clone());
    let paths = isr::page_paths(&legal::hosted_paths(&legal));

    // Before the serve loop, and that ordering is the whole of the handover: the framework's
    // logger installs its own subscriber unless one is already set, and a Sentry layer has to be
    // a layer *of* the subscriber rather than something added to a finished one. With
    // `sentry.enabled` off this installs nothing and the framework's subscriber stands.
    //
    // The binding is held for the rest of `serve`. In a release build that ends after a graceful
    // shutdown, and dropping it is what flushes the Sentry transport; see
    // `telemetry::TelemetryGuard`.
    let _telemetry = match telemetry::init(&config.sentry) {
        Ok(guard) => guard,
        Err(err) => config::refuse(&err),
    };

    // Development keeps the framework's serve loop: it is the one that applies `dx serve`'s
    // hot patches, and it diverges.
    #[cfg(debug_assertions)]
    dioxus::serve(move || {
        let config = Arc::clone(&config);
        let legal = legal.clone();
        let paths = paths.clone();
        async move { Ok(router::router(&config, legal, paths)) }
    });

    // A release build serves until it is told to stop, then returns, so `_telemetry` drops and
    // the Sentry client flushes what its transport still holds.
    #[cfg(not(debug_assertions))]
    serve_until_signalled(move || router::router(&config, legal, paths));
}

/// The exit code for a listener that cannot be set up, `EX_OSERR` from `sysexits.h`.
///
/// The address is taken or not bindable, which is the host's problem instead of the
/// configuration's.
#[cfg(not(debug_assertions))]
const EX_OSERR: i32 = 71;

/// Serves `app` until `SIGTERM` or `SIGINT`, then drains in-flight requests and returns.
///
/// It does what `dioxus::serve` does outside development (the framework's logger, a listener on
/// `IP`/`PORT`, `axum::serve`) plus the graceful shutdown that one lacks. `app` builds the router
/// inside the runtime, which is where `dioxus::serve` builds it too.
// The shutdown is not a nicety in this image. The server is PID 1 of a `scratch` container with no
// init process in front of it, and the kernel delivers no signal to a namespace's PID 1 whose
// default action is to terminate unless the process installed a handler for it. Without the
// handler, `SIGTERM` is discarded: every `docker stop` and every pod deletion waits out the whole
// grace period (10 s for Docker, 30 s for Kubernetes) and then `SIGKILL`s the process, cutting off
// whatever requests were in flight.
#[cfg(not(debug_assertions))]
fn serve_until_signalled(app: impl FnOnce() -> Router) {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|err| exit_unavailable("cannot start the async runtime", &err));
    runtime.block_on(async move {
        // A no-op when telemetry has already installed a subscriber, as under `dioxus::serve`.
        dioxus::logger::initialize_default();
        let addr = dioxus::cli_config::fullstack_address_or_localhost();
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .unwrap_or_else(|err| exit_unavailable(&format!("cannot listen on {addr}"), &err));
        tracing::info!("listening on {addr}");
        match axum::serve(listener, app())
            .with_graceful_shutdown(shutdown_signal())
            .await
        {
            Ok(()) => tracing::info!("every in-flight request finished; exiting"),
            Err(err) => tracing::error!("the server stopped with an error: {err}"),
        }
    });
    // Bounded, because a task still parked on the blocking pool would otherwise hold the exit
    // open for as long as it likes, which is the wait this function exists to remove.
    runtime.shutdown_timeout(RUNTIME_SHUTDOWN_TIMEOUT);
}

/// How long the runtime may take to wind down its worker and blocking threads once every
/// connection has closed.
#[cfg(not(debug_assertions))]
const RUNTIME_SHUTDOWN_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// Ends the process with [`EX_OSERR`], naming what could not be set up.
#[cfg(not(debug_assertions))]
fn exit_unavailable(what: &str, err: &dyn std::fmt::Display) -> ! {
    eprintln!("portfolio: {what}: {err}");
    std::process::exit(EX_OSERR)
}

/// Resolves on the first `SIGTERM`, which Docker and the kubelet send, or `SIGINT` (Ctrl-C).
///
/// A signal stream that cannot be installed is logged and then never resolves.
// Waiting forever beats an immediate shutdown: a server that stops the moment it starts is worse
// than one that has to be killed.
#[cfg(all(not(debug_assertions), unix))]
async fn shutdown_signal() {
    use tokio::signal::unix::{SignalKind, signal};

    async fn received(kind: SignalKind, name: &str) {
        match signal(kind) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            Err(err) => {
                tracing::error!("cannot listen for {name}: {err}");
                std::future::pending::<()>().await;
            }
        }
    }

    let name = tokio::select! {
        () = received(SignalKind::terminate(), "SIGTERM") => "SIGTERM",
        () = received(SignalKind::interrupt(), "SIGINT") => "SIGINT",
    };
    tracing::info!("{name} received; no longer accepting connections, draining in-flight requests");
}

/// Resolves on Ctrl-C.
// No image targets a platform without Unix signals; this keeps a local release build compiling
// on one.
#[cfg(all(not(debug_assertions), not(unix)))]
async fn shutdown_signal() {
    if let Err(err) = tokio::signal::ctrl_c().await {
        tracing::error!("cannot listen for Ctrl-C: {err}");
        std::future::pending::<()>().await;
    }
    tracing::info!("Ctrl-C received; no longer accepting connections, draining in-flight requests");
}
