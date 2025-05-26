use sqlx::{MySqlPool, mysql::MySqlPoolOptions};
use std::env;


pub async fn connect() -> MySqlPool {
    dotenvy::dotenv().ok(); // Load environment variables from .env file

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool.");
    db_pool
}