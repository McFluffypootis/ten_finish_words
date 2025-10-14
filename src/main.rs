use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::PgPool;
use tracing_log::LogTracer;
use zero2production::configuration::get_configuration;
use zero2production::startup::*;

use tracing::{Subscriber, subscriber::set_global_default};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};

use zero2production::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let confguration = get_configuration().expect("Failed to read config");

    let connection_pool =
        PgPool::connect(&confguration.database.connection_string().expose_secret())
            .await
            .expect("Failed to connect to Postgres");

    let address = format!("127.0.0.1:{}", confguration.application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind to random port");

    run(listener, connection_pool)?.await
}
