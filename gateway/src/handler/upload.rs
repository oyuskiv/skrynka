use axum::{
    Json, Router,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::handler::request::JsonRequest;

const UPLOAD_ENDPOINT: &str = "/api/upload";

pub fn router() -> Router {
    Router::new().route(UPLOAD_ENDPOINT, post(create_signed_upload_url))
}

async fn create_signed_upload_url(JsonRequest(req): JsonRequest<SignedUrlRequest>) -> Response {
    info!("request: {:?}", req);
    let url = "todo".to_string();
    SignedUrlResponse { url }.into_response()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignedUrlResponse {
    url: String,
}

impl IntoResponse for SignedUrlResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SignedUrlRequest {
    bucket_name: String,
    file_path: String,
    expires_in: u64,
    headers: Option<Vec<String>>,
}
