use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use std::net::TcpListener;

use sqlx::{Connection, Executor, PgConnection, PgPool};
use ten_finish_words::configuration::{DatabaseSettings, get_configuration};
use ten_finish_words::startup::run;
use ten_finish_words::telemetry::{get_subscriber, init_subscriber};
use uuid::Uuid;

use ten_finish_words::routes::WordResponse;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_layer = "info".to_string();
    let subcriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subcriber_name, default_filter_layer, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subcriber_name, default_filter_layer, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");

    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read config");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let connection_pool = configure_database(&configuration.database).await;

    let server = run(listener, connection_pool.clone()).expect("Failed to bind adress");
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("failed to connect to Postgres");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    sqlx::query!(
        r#"
    INSERT INTO words (id, word, translation ,word_type)
    VALUES ($1, 'yksi', 'one', 'noun'),
           ($2, 'kaksi', 'two', 'noun'),
           ($3, 'kolme', 'three', 'noun'),
           ($4, 'neljä', 'four', 'noun'),
           ($5, 'viisi', 'five', 'noun'),
           ($6, 'kuusi', 'six', 'noun'),
           ($7, 'seitsemän', 'seven', 'noun'),
           ($8, 'kahdeksan', 'eight', 'noun'),
           ($9, 'yhdeksan', 'nine', 'noun'),
           ($10, 'kymenen', 'ten', 'noun');
            
        "#,
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4(),
        Uuid::new_v4()
    )
    .execute(&connection_pool)
    .await
    .expect("Failed to insert test values");

    connection_pool
}

#[tokio::test]
async fn health_check_works() {
    let test_app = spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/health_check", &test_app.await.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscriptions");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request with payload {}",
            error_message
        );
    }
}

#[tokio::test]
async fn get_words_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(&format!("{}/words", &app.address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());

    assert_ne!(Some(0), response.content_length());

    let json_bod = response
        .json::<WordResponse>()
        .await
        .expect("Failed to desearialize words JSON reponse");

    assert_eq!("yksi".to_string(), json_bod.words[0].word);
    assert_eq!("one".to_string(), json_bod.words[0].translation);
}
