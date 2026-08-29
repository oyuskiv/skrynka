mod health;
mod request;
mod upload;

use axum::Router;

pub fn router() -> Router {
    Router::new()
        .merge(health::router())
        .merge(upload::router())
}
