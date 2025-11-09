use axum::{Router, routing::get};
use tokio::net::TcpListener;
use tracing::{Instrument, info};

use crate::error::Error;

pub async fn run(addr: String) -> Result<(), Error> {
    info!("start listening: {}", addr);
    let listener = TcpListener::bind(addr.clone()).await?;
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));
    axum::serve(listener, app)
        .with_graceful_shutdown(graceful_shutdown_signal().in_current_span())
        .await?;
    info!("stop listening: {}", addr);
    Ok(())
}

async fn graceful_shutdown_signal() {
    let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
        .expect("could not register SIGINT signal");
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("could not register SIGTERM signal");
    tokio::select! {
        _ = sigint.recv() => {
            info!("received SIGINT signal initiating graceful shutdown")
        }
        _ = sigterm.recv() => {
            info!("received SIGTERM signal initiating graceful shutdown")
        }
    }
}
