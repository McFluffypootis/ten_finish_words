use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::PgPool;
use ten_finish_words::configuration::get_configuration;
use ten_finish_words::startup::*;
use tracing_log::LogTracer;

use tracing::{Subscriber, subscriber::set_global_default};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt};

use ten_finish_words::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("ten_finish_words".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let confguration = get_configuration().expect("Failed to read config");

    let connection_pool =
        PgPool::connect_lazy(&confguration.database.connection_string().expose_secret())
            .expect("Failed to connect to Postgres");

    let address = format!(
        "{}:{}",
        confguration.application.host, confguration.application.port
    );
    let listener = TcpListener::bind(address).expect("Failed to bind to random port");

    run(listener, connection_pool)?.await
}
