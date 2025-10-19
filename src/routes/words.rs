use actix_web::HttpResponse;

pub async fn words() -> HttpResponse {
    HttpResponse::Ok().finish()
}
