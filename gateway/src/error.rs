use std::{collections, fmt, io};

use serde::{Deserialize, Serialize};

use axum::{
    Json, http,
    response::{IntoResponse, Response},
};

/// The `@type` identifier for an `ErrorInfo` detail payload, matching the `google.rpc.ErrorInfo`
/// proto used across Google APIs.
const ERROR_INFO_TYPE_URL: &str = "type.googleapis.com/google.rpc.ErrorInfo";

/// Structured, machine-readable error details.
///
/// `ErrorInfo` provides standard error payloads containing both human-readable
/// messages for debugging and stable identifiers designed for programmatic handling.
///
/// Its [`IntoResponse`] impl renders it as a Google-style error response, following
///
/// ```json
/// {
///   "error": {
///     "code": 400,
///     "message": "missing field `email`",
///     "status": "INVALID_ARGUMENT",
///     "details": [
///       {
///         "@type": "type.googleapis.com/google.rpc.ErrorInfo",
///         "reason": "INVALID_HTTP_REQUEST",
///         "domain": "gateway.skrynka.dev",
///         "metadata": { "field": "email" }
///       }
///     ]
///   }
/// }
/// ```
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ErrorInfo {
    /// HTTP status code or domain-specific numeric error code (e.g., `404`, `4001`).
    code: u16,

    /// Human-readable error message explaining what went wrong.
    ///
    /// This message is intended for logging or developer debugging and should not
    /// be parsed programmatically.
    message: String,

    /// A stable, machine-readable identifier specifying the exact rule violation.
    ///
    /// Formatted in `SCREAMING_SNAKE_CASE` (e.g., `"INVALID_USER_ID"`, `"RATE_LIMIT_EXCEEDED"`).
    /// Clients should rely on this field for conditional logic rather than matching on [`message`](Self::message).
    reason: String,

    /// The logical grouping to which the "reason" belongs.
    ///
    ///The error domain is typically the registered service name of the tool or product that
    ///generates the error. The error domain must be a globally unique value that identifies the
    ///infrastructure.
    domain: String,

    /// Key-value pairs providing additional contextual metadata about the error.
    ///
    /// Common keys include affected fields, system limits, or resource identifiers
    /// (e.g., `{"field": "email"}`).
    metadata: collections::HashMap<String, String>,
}

impl ErrorInfo {
    /// Default fallback reason when the exact cause of an error cannot be determined
    /// or mapped to a more specific error category.
    ///
    /// Used for unhandled internal failures or unexpected panics.
    pub const REASON_UNKNOWN: &str = "UNKNOWN";

    /// Indicates that a configuration setting is missing, malformed, or invalid.
    ///
    /// Common causes include failing startup validation, out-of-range environment
    /// variables, or bad configuration files (e.g., an invalid connection string).
    pub const REASON_INVALID_CONFIG: &str = "INVALID_CONFIG";

    /// Indicates an underlying operating system or file/network I/O failure.
    ///
    /// Common causes include missing files, permission errors, closed network sockets,
    /// or disk read/write errors.
    pub const REASON_IO_ERROR: &str = "IO_ERROR";

    /// Indicates that an incoming HTTP request violated contract specifications.
    ///
    /// Common causes include missing required headers, malformed query parameters,
    /// or exceeding request payload size limits.
    pub const REASON_INVALID_HTTP_REQUEST: &str = "INVALID_HTTP_REQUEST";

    /// Creates a new `ErrorInfo` with the given message and default fallback fields.
    ///
    /// Default values set:
    /// - `code`: `0`
    /// - `domain`: Empty [`String`]
    /// - `reason`: [`REASON_UNKNOWN`](Self::REASON_UNKNOWN)
    /// - `metadata`: Empty [`HashMap`]
    pub fn new(message: &str) -> Self {
        Self {
            code: 0,
            domain: String::new(),
            reason: ErrorInfo::REASON_UNKNOWN.to_string(),
            message: message.to_string(),
            metadata: collections::HashMap::new(),
        }
    }

    /// Sets the numeric error or HTTP status code.
    pub fn with_code(mut self, code: u16) -> Self {
        self.code = code;
        self
    }

    /// Sets the machine-readable error reason identifier (e.g., [`REASON_INVALID_CONFIG`](Self::REASON_INVALID_CONFIG)).
    pub fn with_reason(mut self, reason: &str) -> Self {
        self.reason = reason.to_string();
        self
    }

    /// Sets logical grouping to which the "reason" belongs.
    pub fn with_domain(mut self, domain: &str) -> Self {
        self.reason = domain.to_string();
        self
    }

    /// Sets the contextual metadata key-value map.
    pub fn with_metadata(mut self, md: collections::HashMap<String, String>) -> Self {
        self.metadata = md;
        self
    }
}

impl IntoResponse for ErrorInfo {
    fn into_response(self) -> Response {
        let status = http::StatusCode::from_u16(self.code)
            .unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR);

        let response = ErrorResponse {
            error: ErrorBody {
                code: status.as_u16(),
                message: &self.message,
                status: canonical_status(status),
                details: [ErrorDetail {
                    type_url: ERROR_INFO_TYPE_URL,
                    reason: &self.reason,
                    domain: &self.domain,
                    metadata: &self.metadata,
                }],
            },
        };

        (status, Json(response)).into_response()
    }
}

impl fmt::Display for ErrorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<envconfig::Error> for ErrorInfo {
    fn from(err: envconfig::Error) -> Self {
        ErrorInfo::new(&err.to_string()).with_reason(ErrorInfo::REASON_INVALID_CONFIG)
    }
}

impl From<io::Error> for ErrorInfo {
    fn from(err: io::Error) -> Self {
        ErrorInfo::new(&err.to_string()).with_reason(ErrorInfo::REASON_IO_ERROR)
    }
}

/// Maps an HTTP status code to its canonical `google.rpc.Code` name.
fn canonical_status(status: http::StatusCode) -> &'static str {
    match status.as_u16() {
        200 => "OK",
        400 => "INVALID_ARGUMENT",
        401 => "UNAUTHENTICATED",
        403 => "PERMISSION_DENIED",
        404 => "NOT_FOUND",
        409 => "ALREADY_EXISTS",
        429 => "RESOURCE_EXHAUSTED",
        499 => "CANCELLED",
        500 => "INTERNAL",
        501 => "UNIMPLEMENTED",
        503 => "UNAVAILABLE",
        504 => "DEADLINE_EXCEEDED",
        _ => "UNKNOWN",
    }
}

/// A single entry in an error response's `details` array.
///
/// Mirrors the `google.rpc.ErrorInfo` message: a stable `reason` plus the `domain` of the
/// service that produced it, and free-form `metadata`.
#[derive(Debug, Clone, Serialize)]
struct ErrorDetail<'a> {
    #[serde(rename = "@type")]
    type_url: &'static str,
    reason: &'a str,
    domain: &'a str,
    metadata: &'a collections::HashMap<String, String>,
}

/// The `google.rpc.Status`- shaped body of an error response.
#[derive(Debug, Clone, Serialize)]
struct ErrorBody<'a> {
    code: u16,
    message: &'a str,
    status: &'static str,
    details: [ErrorDetail<'a>; 1],
}

/// Top-level error response (envelope: `{"error": {...}}`).
#[derive(Debug, Clone, Serialize)]
struct ErrorResponse<'a> {
    error: ErrorBody<'a>,
}
