use tokio::net::TcpListener;
use tracing::{Instrument, info};

use crate::error::ErrorInfo;
use crate::handler;

pub const SERVICE_NAME: &str = "gateway";

/// Starts the HTTP service on the given network address and begins handling incoming requests.
///
/// Binds a TCP listener to `addr` and runs the Axum server indefinitely until a fatal
/// server error occurs or graceful shutdown(SIGINT, SIGTERM) stops server.
pub async fn run(addr: String) -> Result<(), ErrorInfo> {
    info!("start listening {addr:?}");

    let listener = TcpListener::bind(addr.clone()).await?;
    let router = handler::router();

    axum::serve(listener, router)
        .with_graceful_shutdown(graceful_shutdown_signal().in_current_span())
        .await?;

    info!("stop listening {addr:?}");
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
