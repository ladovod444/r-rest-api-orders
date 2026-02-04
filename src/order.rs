use std::str::FromStr;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use crate::AppState;
use crate::user::User;
// use sqlx::types::Decimal;


// Data models
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Order {
    // id: Uuid, // TODO
    // Order_id: i64, // TODO
    // Order_id: i64, // TODO

    #[sqlx(try_from = "i32")]
    pub user_id: i64,

    order_date: Option<chrono::DateTime<chrono::Utc>>,

    #[sqlx(try_from = "i32")]
    pub total_amount: f64, // use sqlx::types::Decimal?
    pub status: String,
    pub shipping_address: String,
    pub billing_address: String,
    pub payment_method: String,
    pub payment_status: String,
    pub notes: String

}

// TODO Requests ...

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    pub user_id: i64,
    // order_date: Option<chrono::DateTime<chrono::Utc>>,
    pub total_amount: f64, // use sqlx::types::Decimal?
    pub status: String,
    pub shipping_address: String,
    pub billing_address: String,
    pub payment_method: String,
    pub payment_status: String,
    pub notes: String
}


// Endpoint Callbacks
// Create Order
// curl -X POST http://localhost:8080/api/orders \
//   -H "Content-Type: application/json" \
//   -d '{"name": "Iphone", "sku": "iphone1234567tt", "description": "best phone(", "price": 1000, "category_id": 1, "image_url": "https://images/1.webp", "is_available": true}'

pub(crate) async fn create_order(
    data: web::Data<AppState>,
    order_req: web::Json<CreateOrderRequest>,
) -> actix_web::Result<HttpResponse> {
    match sqlx::query_as::<_, Order>(
        "INSERT INTO orders (user_id, total_amount, status, shipping_address, billing_address, payment_method, payment_status, notes) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *"
    )
        .bind(&order_req.user_id)
        .bind(&order_req.total_amount)
        .bind(&order_req.status)
        .bind(&order_req.shipping_address)
        // .bind(&order_req.regular_price)
        .bind(&order_req.billing_address)
        .bind(&order_req.payment_method)
        .bind(&order_req.payment_status)
        .bind(&order_req.notes)
        .fetch_one(&data.db)
        .await
    {
        Ok(order) => Ok(HttpResponse::Created().json(order)),
        Err(e) => {
            // Handle unique constraint violation
            if e.to_string().contains("unique constraint") {
                Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Sku already exists"
                })))
            } else {
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": e.to_string()
                })))
            }
        }
    }
}


// curl http://localhost:8080/api/orders
pub async fn get_orders(data: web::Data<AppState>) -> actix_web::Result<HttpResponse> {

    // SELECT amount::FLOAT8

    // match sqlx::query_as::<_, Order>("SELECT * FROM orders ORDER BY created_at DESC")
    match sqlx::query_as::<_, Order>("SELECT order_id, name, sku, description, price::FLOAT8, category_id::INT2, image_url, created_at, updated_at, is_available FROM orders ORDER BY created_at DESC")
        .fetch_all(&data.db)
        .await
    {
        Ok(orders) => Ok(HttpResponse::Ok().json(orders)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))),
    }
}


// Вариант 1: Если Decimal передается как строка
// fn deserialize_decimal_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     // Пробуем десериализовать как строку
//     let s = String::deserialize(deserializer)?;
//     Decimal::from_str(&s)
//         .map(|d| d.to_f64().unwrap_or(0.0))
//         .map_err(serde::de::Error::custom)
// }

fn deserialize_number_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    // Пробуем как f64 напрямую
    let num = f64::deserialize(deserializer)?;
    Ok(num)
}