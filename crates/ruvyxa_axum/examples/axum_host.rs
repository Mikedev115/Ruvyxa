//! One binary: an axum API and the Ruvyxa site it serves.
//!
//! Written into a project as `src/main.rs` by `ruvyxa adds axum`; the same file
//! is this crate's example, so `cargo test` compiles what the scaffold emits.
//!
//! ```text
//! ruvyxa build            # with `adapter: staticOutput()` in ruvyxa.config.ts
//! cargo run               # serves .ruvyxa/static and /api/health on port 3000
//! ```
//!
//! `RUVYXA_STATIC_DIR` names the publish directory, `PORT` and `HOST` the
//! socket — the same variables a container platform sets.

use std::net::SocketAddr;
use std::path::PathBuf;

use axum::routing::get;
use axum::{Json, Router};
use ruvyxa_axum::StaticSite;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let publish_dir = std::env::var_os("RUVYXA_STATIC_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(".ruvyxa/static"));
    let port: u16 = match std::env::var("PORT") {
        Ok(value) => value.parse()?,
        Err(_) => 3000,
    };
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_owned());

    // Reads `_headers` and checks for `404.html` once; fails here, not on the
    // first request, when the build has not run.
    let site = StaticSite::open(&publish_dir)?;

    let app = Router::new()
        .route("/api/health", get(health))
        // Routes above this line answer with the same security headers as the
        // published files. `layer` wraps only what was added before it, which
        // is why the site's own router — already carrying the policy — is
        // mounted afterwards.
        .layer(site.headers_layer())
        // Every path the API does not answer: pages, assets, the site's 404.
        .fallback_service(site.router());

    let address: SocketAddr = format!("{host}:{port}").parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!(
        "serving {} on http://{}",
        site.publish_dir().display(),
        listener.local_addr()?
    );
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok" }))
}
