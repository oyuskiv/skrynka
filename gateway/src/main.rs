mod built {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use std::process::ExitCode;

use tracing::{Instrument, Level, error, info, span};

use gateway::config::{Config, LogFormat};
use gateway::error;
use gateway::service;

#[tokio::main]
async fn main() -> ExitCode {
    let cfg = match Config::init_from_env() {
        Ok(cfg) => cfg,
        Err(err) => {
            // No subscriber yet — stderr is the only channel that works here.
            eprintln!("invalid configuration: {err}");
            return ExitCode::FAILURE;
        }
    };

    match run(cfg).await {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => {
            error!(error = %err, "service unexpectedly stopped");
            return ExitCode::FAILURE;
        }
    }
}

async fn run(cfg: Config) -> Result<(), error::ErrorInfo> {
    let subscriber = tracing_subscriber::fmt::SubscriberBuilder::default()
        .with_env_filter(cfg.tracing_level)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_target(true)
        .with_level(true);

    match cfg.format_log {
        LogFormat::Json => subscriber.json().init(),
        LogFormat::Ansi => subscriber.with_ansi(true).init(),
    };

    let commit = built::GIT_COMMIT_HASH.unwrap_or("n/a");
    let built_time = built::BUILT_TIME_UTC;
    info!("start service (commit={commit:?}, built time={built_time:?})",);

    service::run(cfg.addr)
        .instrument(span!(parent: None, Level::INFO, "service"))
        .await?;

    info!("service stopped");
    Ok(())
}
