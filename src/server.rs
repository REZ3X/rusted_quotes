use axum::{
    extract::{ Path, Query, State },
    http::{ Method, StatusCode },
    response::Json,
    routing::{ get, post },
    Router,
};
use serde::{ Deserialize, Serialize };
use sqlx::{ mysql::MySqlPool, Row };
use std::{ env, sync::Arc };
use tower_http::cors::{ Any, CorsLayer };
use uuid::Uuid;
use chrono::{ DateTime, Utc };

#[derive(Debug, Serialize, Deserialize)]
struct Quote {
    id: String,
    quote: String,
    author: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateQuote {
    quote: String,
    author: Option<String>,
}

#[derive(Debug, Deserialize)]
struct QuoteQuery {
    page: Option<u64>,
    limit: Option<u64>,
    search: Option<String>,
}

type AppState = Arc<MySqlPool>;

fn contains_inappropriate_content(text: &str) -> bool {
    let inappropriate_words = env
        ::var("INAPPROPRIATE_WORDS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .collect::<Vec<_>>();

    let text_lower = text.to_lowercase();
    inappropriate_words.iter().any(|word| { !word.is_empty() && text_lower.contains(word) })
}

async fn create_quote(
    State(pool): State<AppState>,
    Json(payload): Json<CreateQuote>
) -> Result<Json<Quote>, StatusCode> {
    if
        contains_inappropriate_content(&payload.quote) ||
        payload.author.as_ref().map_or(false, |a| contains_inappropriate_content(a))
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let result = sqlx
        ::query(
            "INSERT INTO quotes (id, quote, author, created_at, updated_at) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(&payload.quote)
        .bind(&payload.author)
        .bind(&now)
        .bind(&now)
        .execute(&*pool).await;

    match result {
        Ok(_) => {
            let quote = Quote {
                id,
                quote: payload.quote,
                author: payload.author,
                created_at: now,
                updated_at: now,
            };
            Ok(Json(quote))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_quotes(
    State(pool): State<AppState>,
    Query(params): Query<QuoteQuery>
) -> Result<Json<Vec<Quote>>, StatusCode> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(10).min(100);
    let offset = (page - 1) * limit;

    let quotes = if let Some(search) = params.search {
        sqlx
            ::query(
                "SELECT id, quote, author, created_at, updated_at FROM quotes 
             WHERE quote LIKE ? OR author LIKE ? 
             ORDER BY created_at DESC 
             LIMIT ? OFFSET ?"
            )
            .bind(format!("%{}%", search))
            .bind(format!("%{}%", search))
            .bind(limit)
            .bind(offset)
            .fetch_all(&*pool).await
    } else {
        sqlx
            ::query(
                "SELECT id, quote, author, created_at, updated_at FROM quotes 
             ORDER BY created_at DESC 
             LIMIT ? OFFSET ?"
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(&*pool).await
    };

    match quotes {
        Ok(rows) => {
            let quotes: Vec<Quote> = rows
                .into_iter()
                .map(|row| Quote {
                    id: row.get("id"),
                    quote: row.get("quote"),
                    author: row.get("author"),
                    created_at: row.get("created_at"),
                    updated_at: row.get("updated_at"),
                })
                .collect();
            Ok(Json(quotes))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_random_quote(State(pool): State<AppState>) -> Result<Json<Quote>, StatusCode> {
    let result = sqlx
        ::query(
            "SELECT id, quote, author, created_at, updated_at FROM quotes ORDER BY RAND() LIMIT 1"
        )
        .fetch_optional(&*pool).await;

    match result {
        Ok(Some(row)) => {
            let quote = Quote {
                id: row.get("id"),
                quote: row.get("quote"),
                author: row.get("author"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Json(quote))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Database error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_quote_by_id(
    State(pool): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>
) -> Result<Json<Quote>, StatusCode> {
    println!("Attempting to fetch quote with ID: {}", id);

    let result = sqlx
        ::query("SELECT id, quote, author, created_at, updated_at FROM quotes WHERE id = ?")
        .bind(&id)
        .fetch_optional(&*pool).await;

    match result {
        Ok(Some(row)) => {
            println!("Found quote: {}", id);
            let quote = Quote {
                id: row.get("id"),
                quote: row.get("quote"),
                author: row.get("author"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Json(quote))
        }
        Ok(None) => {
            println!("Quote not found: {}", id);
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    println!("Connecting to database...");
    let pool = MySqlPool::connect(&database_url).await.expect("Failed to connect to MySQL");

    println!("Creating/verifying database table...");
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS quotes (
            id VARCHAR(36) PRIMARY KEY,
            quote TEXT NOT NULL,
            author VARCHAR(255),
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
        )
        "#
    )
        .execute(&pool).await
        .expect("Failed to create quotes table");

    println!("Database table created/verified");

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/quotes/random", get(get_random_quote))
        .route("/api/quotes/:id", get(get_quote_by_id))
        .route("/api/quotes", get(get_quotes).post(create_quote))
        .layer(cors)
        .with_state(Arc::new(pool));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server running on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
