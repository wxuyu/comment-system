//! Vercel serverless function entry point.
//!
//! On Linux (the Vercel build target) this uses the official `@vercel/rust`
//! runtime (`vercel_runtime` 2.x) together with its `axum::VercelLayer`
//! adapter: the layer converts Vercel's `(AppState, Request)` into an axum
//! `Request<Body>` and converts the axum response back into Vercel's
//! `Response<ResponseBody>`. All business logic lives in `artalk-server` /
//! `artalk-core`; this file is a thin adapter.
//!
//! On non-Linux platforms (e.g. local Windows dev) `vercel_runtime` cannot
//! compile, so we provide a stub `main` that points to `cargo run --bin serve`.

#[cfg(target_os = "linux")]
mod vercel_main {
    use artalk_server::{build_app, router};
    use tower::{ServiceBuilder, ServiceExt};
    use vercel_runtime::axum::VercelLayer;
    use vercel_runtime::{run, service_fn, AppState, Error, Request};

    pub async fn run_vercel() -> Result<(), Error> {
        let app_state = build_app().await?;
        let axum_router = router(app_state);
        let svc = ServiceBuilder::new()
            .layer(VercelLayer::new())
            .service(axum_router);

        run(service_fn(move |(state, req): (AppState, Request)| {
            let mut svc = svc.clone();
            async move {
                ServiceExt::<(AppState, Request)>::ready(&mut svc)
                    .await
                    .map_err(|e| Box::new(e) as Error)?;
                svc.call((state, req))
                    .await
                    .map_err(|e| Box::new(e) as Error)
            }
        }))
        .await
    }
}

#[cfg(target_os = "linux")]
fn main() -> Result<(), vercel_runtime::Error> {
    // A Tokio runtime is provided by vercel_runtime's `run`; we still need a
    // #[tokio::main]-style entry. vercel_runtime's `run` blocks on the runtime
    // it manages, so we call it directly.
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(vercel_main::run_vercel())
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!(
        "This `api` binary is the Vercel serverless entry and only builds on Linux.\n\
         For local development run: cargo run --bin serve"
    );
    std::process::exit(1);
}
