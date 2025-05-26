mod db;
mod models;
mod handlers;

use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use handlers::user::{register_user, get_users, login_user};


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok(); // Load environment variables from .env file

    let db_pool = db::connect().await; // Connect to the database

    println!("Connected to the database");

    // Create the database table if it doesn't exist in database

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS users (
            id VARCHAR(36) PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            email VARCHAR(255) NOT NULL UNIQUE
        )",
    )
    .execute(&db_pool)
    .await
    .expect("Failed to create table");
    println!("Database table created");

    println!("Starting server at http://127.0.1:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone())) // Pass the database pool to the app
            .route("/register", web::post().to(register_user))
            .route("/users", web::get().to(get_users))
            .route("/login", web::post().to(login_user))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}