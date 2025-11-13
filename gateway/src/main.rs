mod built {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use envconfig::Envconfig;
use tracing::{Instrument, Level, info, span};

use gateway::app;
use gateway::config::{Config, LogFormat};
use gateway::error::Error;

#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => (),
        Err(err) => {
            eprintln!("error: {}", err);
            std::process::exit(1);
        }
    }
}

async fn run() -> Result<(), Error> {
    let cfg = Config::init_from_env()?;

    let subsciber = tracing_subscriber::fmt::SubscriberBuilder::default()
        .with_env_filter(cfg.tracing_level)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_target(true)
        .with_level(true);
    match cfg.format_log {
        LogFormat::Json => subsciber.json().init(),
        LogFormat::Ansi => subsciber.with_ansi(true).init(),
    };

    info!(
        "start service: {}, commit hash: {}, build time: {}",
        built::PKG_NAME,
        built::GIT_COMMIT_HASH.unwrap_or("n/a"),
        built::BUILT_TIME_UTC
    );

    app::run(cfg.addr)
        .instrument(span!(parent: None, Level::INFO, "app"))
        .await?;

    info!("service {} stopped", built::PKG_NAME);
    Ok(())
}
