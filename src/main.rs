use std::net::TcpListener;

use zero2production::routes;
use zero2production::startup::*;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");

    let port = listener.local_addr().unwrap().port();

    run(listener)?.await
}
