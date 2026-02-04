use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::AppState;

// Data models
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    // id: Uuid,
    // product_id: i64, // TODO
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
    pub address: String,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
    is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    // product_id: i64, // TODO
    pub username: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
    pub address: String,
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    updated_at: Option<chrono::DateTime<chrono::Utc>>, // TIMESTAMPTZ
    is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
}

// Endpoint callbacks
// Get all users
// curl http://localhost:8080/api/users
pub async fn get_users(data: web::Data<AppState>) -> actix_web::Result<HttpResponse> {
    match sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
        .fetch_all(&data.db)
        .await
    {
        Ok(users) => Ok(HttpResponse::Ok().json(users)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))),
    }
}

// Get user by ID
pub async fn get_user(
    data: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> actix_web::Result<HttpResponse> {
    let user_id = path.into_inner();

    match sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&data.db)
        .await
    {
        Ok(Some(user)) => Ok(HttpResponse::Ok().json(user)),
        Ok(None) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "User not found"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))),
    }
}

// Create new user
// curl -X POST http://localhost:8080/api/users \
//   -H "Content-Type: application/json" \
//   -d '{"name": "John Doe", "email": "john@example.com"}'

// curl -X POST http://localhost:8080/api/users -H "Content-Type: application/json" -d '{"name": "Jim Beam", "email": "beam@example.com"}'
pub async fn create_user(
    data: web::Data<AppState>,
    user_req: web::Json<CreateUserRequest>,
) -> actix_web::Result<HttpResponse> {
    match sqlx::query_as::<_, User>(
        "INSERT INTO users (username, email, first_name, last_name, phone, address, is_active) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *"
    )
        .bind(&user_req.username)
        .bind(&user_req.email)
        .bind(&user_req.first_name)
        .bind(&user_req.last_name)
        .bind(&user_req.phone)
        .bind(&user_req.address)
        .bind(&user_req.is_active)
        .fetch_one(&data.db)
        .await
    {
        Ok(user) => Ok(HttpResponse::Created().json(user)),
        Err(e) => {
            // Handle unique constraint violation
            if e.to_string().contains("unique constraint") {
                Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Email already exists"
                })))
            } else {
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": e.to_string()
                })))
            }
        }
    }
}

// Update user
pub async fn update_user(
    data: web::Data<AppState>,
    path: web::Path<Uuid>,
    update_req: web::Json<UpdateUserRequest>,
) -> actix_web::Result<HttpResponse> {
    let user_id = path.into_inner();

    let mut query = "UPDATE users SET ".to_string();
    let mut params = Vec::new();
    let mut param_count = 1;

    if let Some(name) = &update_req.name {
        query.push_str(&format!("name = ${} ", param_count));
        params.push(name);
        param_count += 1;
    }

    if let Some(email) = &update_req.email {
        if param_count > 1 {
            query.push_str(", ");
        }
        query.push_str(&format!("email = ${} ", param_count));
        params.push(email);
        param_count += 1;
    }

    query.push_str(&format!("WHERE id = ${} RETURNING *", param_count));
    params.push(&user_id.to_string());

    // For simplicity, using a prepared approach
    match sqlx::query_as::<_, User>(
        "UPDATE users SET name = COALESCE($1, name), email = COALESCE($2, email) WHERE id = $3 RETURNING *"
    )
        .bind(&update_req.name)
        .bind(&update_req.email)
        .bind(user_id)
        .fetch_optional(&data.db)
        .await
    {
        Ok(Some(user)) => Ok(HttpResponse::Ok().json(user)),
        Ok(None) => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "User not found"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))),
    }
}

// Delete user
pub async fn delete_user(
    data: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> actix_web::Result<HttpResponse> {
    let user_id = path.into_inner();

    match sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&data.db)
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                Ok(HttpResponse::NoContent().finish())
            } else {
                Ok(HttpResponse::NotFound().json(serde_json::json!({
                    "error": "User not found"
                })))
            }
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))),
    }
}
