use actix_web::{HttpResponse, mime};
use serde::{Deserialize, Serialize};

/*
 {
    "words": [words1, word2, ... words10]
  }
*/

#[derive(Serialize, Deserialize)]
pub struct WordResponse {
    pub message: String,
}

pub async fn words() -> HttpResponse {
    let response = WordResponse {
        message: "hello".to_string(),
    };

    HttpResponse::Ok().json(response)
}
