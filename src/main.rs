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

/*
*
* CREATE TABLE subscriptions(
id uuid NOT NULL,
PRIMARY KEY (id),
email TEXT NOT NULL UNIQUE,
name TEXT NOT NULL,
subscribed_at timestamptz NOT NULL
);

CREATE TABLE words(
id uuid NOT NULL,
PRIMARY KEY (id),
word TEXT NOT NULL UNIQUE,
translation TEXT NOT NULL,
word_type TEXT NOT NULL,
access_count INTEGER NOT NULL DEFAULT 0
);
*/

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = get_subscriber("ten_finish_words".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let confguration = get_configuration().expect("Failed to read config");

    let connection_pool = PgPool::connect_lazy_with(confguration.database.with_db());

    let address = format!(
        "{}:{}",
        confguration.application.host, confguration.application.port
    );
    let listener = TcpListener::bind(address).expect("Failed to bind to random port");

    run(listener, connection_pool)?.await
}
