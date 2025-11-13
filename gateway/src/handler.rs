mod health;

use axum::{Router, routing::get};

pub fn create_health_router() -> Router {
    Router::new().route("/health", get(health::get_health))
}
