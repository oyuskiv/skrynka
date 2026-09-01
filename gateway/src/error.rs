use std::{collections, fmt, io};

use serde::{Deserialize, Serialize};

use crate::generated;

/// Structured, machine-readable error details.
///
/// `ErrorInfo` provides standard error payloads containing both human-readable
/// messages for debugging and stable identifiers designed for programmatic handling.
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
    metadata: collections::BTreeMap<String, String>,
}

impl ErrorInfo {
    /// Default fallback reason when the exact cause of an error cannot be determined
    /// or mapped to a more specific error category.
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
            metadata: collections::BTreeMap::new(),
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
        self.domain = domain.to_string();
        self
    }

    /// Sets the contextual metadata key-value map.
    pub fn with_metadata(mut self, md: collections::BTreeMap<String, String>) -> Self {
        self.metadata = md;
        self
    }
}

impl From<ErrorInfo> for generated::Error {
    fn from(err: ErrorInfo) -> Self {
        Self {
            error: generated::Status {
                code: Option::Some(err.code as i64),
                message: Option::Some(err.message),
                details: Option::Some(vec![generated::StatusDetail::ErrorInfo(
                    generated::ErrorInfo {
                        domain: err.domain,
                        metadata: Option::Some(generated::ErrorInfoMetadata {
                            additional_properties: err.metadata,
                        }),
                        reason: err.reason,
                    },
                )]),
            },
        }
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
