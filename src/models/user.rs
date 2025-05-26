use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,

    #[validate(email(message = "Invalid email address"))]
    pub email: String,

    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}


#[derive(Debug, sqlx::FromRow, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}