use anyhow::{Context, Error, Result};
use axum::Server;
use axum::{routing::get, Router};
use config::{Config, Environment, File};
use serde::Deserialize;
use std::env;
use std::error::Error as StdError;
use std::net::{IpAddr, SocketAddr};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::{debug, error};

const ENVIRONMENT: &str = "ENVIRONMENT";

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub addr: IpAddr,
    pub port: u16,
}

impl Settings {
    /// First the file `config/default` is read, then the file `config/<ENVIRONMENT>`,
    /// e.g. `config/dev`, if the environment variable `ENVIRONMENT` is defined,
    /// and finally environment variables prefixed with `APP__` and separated by `__`
    /// (double underscores are used as separators because of snake_cased keys).
    fn new() -> Result<Self> {
        env::var(ENVIRONMENT)
            .iter()
            .fold(
                Config::builder().add_source(File::with_name("config/default")),
                |config, env| config.add_source(File::with_name(&format!("config/{env}"))),
            )
            .add_source(Environment::with_prefix("app").separator("__"))
            .build()?
            .try_deserialize()
            .context("Error creating configuration settings")
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .json()
        .init();

    if let Err(e) = run().await {
        log_error("bayer-axum exited with ERROR", e);
    };
}

async fn run() -> Result<()> {
    let settings = Settings::new()?;
    debug!("Starting with these settings: {settings:?}");

    let app = Router::new()
        .route("/", get(|| async { "Habe die Ehre!" }))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let addr = SocketAddr::new(settings.addr, settings.port);
    tokio::spawn(async move { Server::bind(&addr).serve(app.into_make_service()).await })
        .await
        .map(|server_result| server_result.context("Server completed with error"))
        .context("Server panicked")
        .and_then(|r| r)
}

fn log_error(message: &str, e: Error) {
    let mut error_chain = Vec::new();
    build_error_chain(&mut error_chain, e.source());
    match error_chain[..] {
        [] => error!(message = message, error = display(&e)),
        [s] => error!(message = message, error = display(&e), source = display(&s)),
        [s1, s2] => error!(
            message = message,
            error = display(&e),
            source = display(&s1),
            source2 = display(&s2),
        ),
        [s1, s2, s3, ..] => error!(
            message = message,
            error = display(&e),
            source = display(&s1),
            source2 = display(&s2),
            source3 = display(&s3),
        ),
    }
}

fn build_error_chain<'a>(chain: &mut Vec<&'a (dyn StdError)>, e: Option<&'a (dyn StdError)>) {
    if let Some(e) = e {
        chain.push(e);
        build_error_chain(chain, e.source());
    }
}
