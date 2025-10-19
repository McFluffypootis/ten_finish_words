use actix_web::{HttpResponse, web};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/*
 {
    "words": [words1, word2, ... words10]
  }
*/

#[derive(Serialize, Deserialize)]
pub struct WordResponse {
    pub message: String,
    pub words: Vec<String>,
}

#[tracing::instrument(name = "Responding with todays words", skip(pool))]
pub async fn words(pool: web::Data<PgPool>) -> HttpResponse {
    match select_words(&pool).await {
        Ok(words) => {
            let response = WordResponse {
                message: "Hei".to_string(),
                words: words,
            };
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            tracing::error!(" Failed to execute query: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

// select 10 rows (at random?)(words + translation) with lowest pick score
// Increment the pick score by one for these rows
// return rows

#[tracing::instrument(name = "Selecting words from database", skip(pool))]
pub async fn select_words(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
    UPDATE words 
    SET access_count = access_count +1 
    WHERE word IN (
        SELECT word FROM words 
        ORDER BY access_count DESC
        LIMIT $1
        FOR UPDATE SKIP LOCKED
    ) 
    RETURNING word;
    "#,
        10
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query {:?}", e);
        e
    })?;

    Ok(result.into_iter().map(|w| w.word.to_string()).collect())
}
