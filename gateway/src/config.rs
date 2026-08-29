use crate::{error, service};
use envconfig::Envconfig;
use std::collections;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[derive(Envconfig)]
struct Env {
    #[envconfig(from = "SKRYNKA_GATEWAY_ADDRESS", default = "127.0.0.1:2345")]
    addr: String,
    #[envconfig(from = "SKRYNKA_GATEWAY_TRACING", default = "info")]
    tracing_level: String,
    #[envconfig(from = "SKRYNKA_GATEWAY_LOG_FORMAT", default = "ansi")]
    format_log: String,
}

/// Service configuration parameters.
pub struct Config {
    /// Network address and port for the server to bind to (e.g., `"127.0.0.1:2345"`).
    pub addr: String,
    /// Minimum log verbosity filter level.
    pub tracing_level: TracingLevel,
    /// Output formatting style used by the logger.
    pub format_log: LogFormat,
}

impl Config {
    /// Loads and parses service configuration from environment variables.
    pub fn init_from_env() -> Result<Self, error::ErrorInfo> {
        let raw = Env::init_from_env()?;
        Ok(Self {
            addr: raw.addr,
            tracing_level: raw.tracing_level.parse()?,
            format_log: raw.format_log.parse()?,
        })
    }
}

/// Output format choices for service log events.
pub enum LogFormat {
    /// Human-readable text output with ANSI color codes, optimized for local development terminals.
    Ansi,
    /// Structured JSON output, optimized for log collection agents and aggregation services (e.g., Datadog, ELK).
    Json,
}

impl std::str::FromStr for LogFormat {
    type Err = error::ErrorInfo;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ansi" => Ok(LogFormat::Ansi),
            "json" => Ok(LogFormat::Json),
            _ => Err(error::ErrorInfo::new("expected ansi or json")
                .with_reason(error::ErrorInfo::REASON_INVALID_CONFIG)
                .with_domain(service::SERVICE_NAME)
                .with_metadata(collections::HashMap::from([(
                    "SKRYNKA_GATEWAY_LOG_FORMAT".to_string(),
                    s.to_string(),
                )]))),
        }
    }
}

/// A wrapper around [`tracing_subscriber::EnvFilter`] that controls log verbosity and target filtering.
///
/// Parses standard directive strings to configure log levels globally or per module/crate
/// (e.g., `"info"`, `"debug"`, or `"my_app=trace,tokio=warn"`).
pub struct TracingLevel(EnvFilter);

impl From<TracingLevel> for EnvFilter {
    fn from(value: TracingLevel) -> Self {
        value.0
    }
}

impl std::str::FromStr for TracingLevel {
    type Err = error::ErrorInfo;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid = |detail: String| {
            error::ErrorInfo::new(&detail)
                .with_reason(error::ErrorInfo::REASON_INVALID_CONFIG)
                .with_domain(service::SERVICE_NAME)
                .with_metadata(collections::HashMap::from([(
                    "SKRYNKA_GATEWAY_TRACING".to_string(),
                    s.to_string(),
                )]))
        };

        if s.trim().is_empty() {
            return Err(invalid("must not be empty".into()));
        }

        for directive in s.split(',').map(str::trim) {
            if !directive.contains('=') && LevelFilter::from_str(directive).is_err() {
                return Err(invalid(format!(
                    "{directive:?} is not a level (expected off|error|warn|info|debug|trace)"
                )));
            }
        }
        EnvFilter::builder()
            .parse(s)
            .map(TracingLevel)
            .map_err(|e| invalid(e.to_string()))
    }
}
