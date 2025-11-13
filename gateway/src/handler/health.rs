use axum::http::StatusCode;
use tracing::{instrument, trace};

#[instrument]
pub async fn get_health() -> StatusCode {
    trace!("Ok");
    StatusCode::OK
}
