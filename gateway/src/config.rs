use crate::error::Error;
use envconfig::Envconfig;
use tracing_subscriber::EnvFilter;

#[derive(Envconfig)]
pub struct Config {
    #[envconfig(from = "SKRYNKA_GATEWAY_ADDRESS", default = "127.0.0.1:2345")]
    pub addr: String,
    #[envconfig(from = "SKRYNKA_GATEWAY_TRACING", default = "info")]
    pub tracing_level: TracingLavel,
    #[envconfig(from = "SKRYNKA_GATEWAY_LOG_FORMAT", default = "ansi")]
    pub format_log: LogFormat,
}

pub enum LogFormat {
    Ansi,
    Json,
}

impl std::str::FromStr for LogFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ansi" => Ok(LogFormat::Ansi),
            "json" => Ok(LogFormat::Json),
            _ => Err(Error::Config("SKRYNKA_GATEWAY_LOG_FORMAT".to_string())),
        }
    }
}

pub struct TracingLavel(EnvFilter);

impl Into<EnvFilter> for TracingLavel {
    fn into(self) -> EnvFilter {
        self.0
    }
}

impl std::str::FromStr for TracingLavel {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match EnvFilter::builder().parse(s) {
            Ok(filter) => Ok(TracingLavel(filter)),
            Err(_) => Err(Error::Config("SKRYNKA_GATEWAY_TRACING".to_string())),
        }
    }
}
