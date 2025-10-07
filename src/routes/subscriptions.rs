use std::net::TcpListener;

use actix_web::{App, HttpResponse, HttpServer, Responder, dev::Server, web};

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(_form: web::Form<FormData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
