use actix_web::{HttpResponse, web};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/*
 {
    "words": [words1, word2, ... words10]
  }
*/
#[derive(Serialize, Deserialize, Debug)]
pub struct WordTranslation {
    pub word: String,
    pub translation: String,
}

#[derive(Serialize, Deserialize)]
pub struct WordResponse {
    pub words: Vec<WordTranslation>,
}

#[tracing::instrument(name = "Responding with todays words", skip(pool))]
pub async fn words(pool: web::Data<PgPool>) -> HttpResponse {
    match select_words(&pool).await {
        Ok(words) => {
            let response = WordResponse { words: words };
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
pub async fn select_words(pool: &PgPool) -> Result<Vec<WordTranslation>, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        SELECT word, translation FROM words 
        WHERE word IN (
            SELECT word FROM words 
            ORDER BY access_count DESC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
        ) 
    "#,
        10
    )
    .fetch_all(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query {:?}", e);
        e
    })?;

    //split into two queries for now but maybe combine into one??
    sqlx::query!(
        r#"
    UPDATE words 
    SET access_count = access_count +1 
    WHERE word IN (
        SELECT word FROM words 
        ORDER BY access_count DESC
        LIMIT $1
        FOR UPDATE SKIP LOCKED
    ) 
    "#,
        10
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query {:?}", e);
        e
    })?;

    Ok(result
        .into_iter()
        .map(|w| WordTranslation {
            word: w.word.to_string(),
            translation: w.translation.to_string(),
        })
        .collect())
}
