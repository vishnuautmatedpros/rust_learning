// Import necessary modules from Actix-Web
use actix_web::{web, HttpResponse, Responder};

// Import MySQL connection pool from SQLx
use sqlx::MySqlPool;

// Import UUID generator for user IDs
use uuid::Uuid;

// Import Argon2 for password hashing and verification
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

// Import helper for generating random salt
use password_hash::SaltString;
use rand::rngs::OsRng; // OS secure random number generator

// Import the `Validate` trait for input validation
use validator::Validate;

// Import application-level models
use crate::models::user::{RegisterRequest, User, LoginRequest};

/// Handler for user registration
pub async fn register_user(
    user: web::Json<RegisterRequest>, // Deserialize and extract the request JSON into a validated RegisterRequest struct
    db: web::Data<MySqlPool>,         // Inject the SQLx MySQL connection pool
) -> impl Responder {
    // 🔍 Validate user input using the validator crate
    if let Err(validation_errors) = user.validate() {
        // If validation fails, serialize the errors into JSON and return a 400 Bad Request
        let error_json = serde_json::to_value(&validation_errors).unwrap();
        return HttpResponse::BadRequest().json(error_json);
    }

    // ✅ Generate a new UUID for the user
    let user_id = Uuid::new_v4();

    // 🔐 Generate a random salt for password hashing
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default(); // Use default Argon2 parameters

    // 🔒 Hash the user's password using Argon2 and the generated salt
    let hashed_password = argon2
        .hash_password(user.password.as_bytes(), &salt)
        .unwrap()
        .to_string(); // Convert the hash to a string to store in DB

    // 🛢️ Insert the new user into the database
    let result = sqlx::query("INSERT INTO users (id, name, email, password) VALUES (?, ?, ?, ?)")
        .bind(user_id.to_string())  // Bind UUID
        .bind(&user.name)           // Bind name
        .bind(&user.email)          // Bind email
        .bind(&hashed_password)     // Bind hashed password
        .execute(db.get_ref())      // Execute query using DB connection
        .await;

    // 📤 Return response based on result
    match result {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({ "message": "User registered successfully" })),
        Err(e) => {
            eprintln!("Error inserting user: {}", e); // Log error
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Something went wrong" }))
        }
    }
}

/// Handler for user login
pub async fn login_user(
    user: web::Json<LoginRequest>, // Deserialize JSON payload into LoginRequest
    db: web::Data<MySqlPool>,      // Inject SQLx connection pool
) -> impl Responder {
    let email = &user.email;
    let password = &user.password;

    // 🔍 Query user by email (and fetch password hash)
    let result = sqlx::query_as::<_, LoginRequest>(
        "SELECT id, name, email, password FROM users WHERE email = ?"
    )
        .bind(email)
        .fetch_one(db.get_ref())
        .await;

    // 🎯 Handle DB result
    match result {
        Ok(user) => {
            // 🔐 Parse stored password hash string into PasswordHash
            let parsed_hash = PasswordHash::new(&user.password).unwrap();

            // ✅ Verify input password against stored hash
            if Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok()
            {
                HttpResponse::Ok().json(serde_json::json!({ "message": "Login successful" }))
            } else {
                HttpResponse::Unauthorized().json(serde_json::json!({ "error": "Invalid password" }))
            }
        }
        Err(e) => {
            eprintln!("Error fetching user: {}", e); // Log error
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Login failed" }))
        }
    }
}

/// Handler to fetch all users (for admin/debug purposes)
pub async fn get_users(db: web::Data<MySqlPool>) -> impl Responder {
    // 🧾 Query all users (omit password for security)
    let users = sqlx::query_as::<_, User>("SELECT id, name, email FROM users")
        .fetch_all(db.get_ref())
        .await;

    // 📤 Return users in JSON or error
    match users {
        Ok(rows) => HttpResponse::Ok().json(rows),
        Err(e) => {
            eprintln!("Error fetching users: {}", e); // Log error
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Could not fetch users" }))
        }
    }
}

pub async fn get_user_by_id(
    user_id: web::Path<String>,
    db: web::Data<MySqlPool>,
) -> impl Responder {
    let result = sqlx::query_as::<_, User>(
        "SELECT id, name, email FROM users WHERE id = ?"
    )
    .bind(user_id.into_inner())
    .fetch_one(db.get_ref())
    .await;

    match result {  
        Ok(user) => HttpResponse::Ok().json(user),
        Err(sqlx::Error::RowNotFound) => {
            HttpResponse::NotFound().json(serde_json::json!({ "error": "User not found" }))
        }
        Err(e) => {
            eprintln!("Error fetching user: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({ "error": "Could not fetch user" }))
        }
    }
}
