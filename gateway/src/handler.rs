use std::sync::Arc;

use async_trait::async_trait;
use axum::Router;
use tracing::{instrument, trace};

use crate::generated;

#[derive(Debug)]
struct Api;

/// The service is healthy and open for requests.
const STATUS_SERVING: &str = "SERVING";

/// The service is unhealthy.
const STATUS_NOT_SERVING: &str = "NOT_SERVING";

#[async_trait]
impl generated::ServerApi for Arc<Api> {
    #[instrument]
    async fn health_probe(&self) -> generated::HealthProbeResponse {
        trace!(STATUS_SERVING);
        generated::HealthProbeResponse::Ok(generated::Status {
            message: Option::Some(STATUS_SERVING.to_string()),
            ..Default::default()
        })
    }
}

pub fn router() -> Router {
    let api = Arc::new(Api);
    generated::server_api_router(api)
}
