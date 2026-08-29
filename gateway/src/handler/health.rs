use axum::http::StatusCode;
use axum::{Router, routing::get};
use tracing::{instrument, trace};

pub fn router() -> Router {
    Router::new().route("/health", get(get_health))
}

#[instrument]
async fn get_health() -> StatusCode {
    trace!("Ok");
    StatusCode::OK
}
