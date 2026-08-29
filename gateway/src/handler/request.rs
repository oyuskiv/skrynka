use axum::{
    extract::{FromRequest, Request},
    response::{IntoResponse, Response},
};

use crate::error;

/// An Axum extractor wrapper for JSON request payloads that automatically converts deserialization
/// failures into structured [`ErrorInfo`] responses.
///
/// It replaces Axum's default plain-text rejections with standardized, machine-readable JSON error
/// bodies.
pub struct JsonRequest<T>(pub T);

impl<S, T> FromRequest<S> for JsonRequest<T>
where
    T: serde::de::DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(req, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => Err(error::ErrorInfo::new(&rejection.body_text().to_lowercase())
                .with_code(rejection.status().as_u16())
                .with_reason(error::ErrorInfo::REASON_INVALID_HTTP_REQUEST)
                .into_response()),
        }
    }
}
