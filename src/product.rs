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
pub struct Product {
    // id: Uuid, // TODO
    // product_id: i64, // TODO
    // product_id: i64, // TODO
    pub name: String,
    pub description: String,
    pub sku: String,


     // #[serde(deserialize_with = "deserialize_number_to_f64")]

    /*
     #[sqlx(try_from = "i32")]
     pub price: f64, // use sqlx::types::Decimal?
     */


    #[sqlx(try_from = "i32")]
    pub stock_quantity: i64,


    #[sqlx(try_from = "i32")]
    pub category_id: i16,
    pub image_url: String,

    // #[sqlx(try_from = "NaiveDateTime")]
    created_at: Option<chrono::DateTime<chrono::Utc>>,

    // #[sqlx(try_from = "NaiveDateTime")]
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
    is_available: bool,
    // deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}

// TODO Requests ...

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub description: String,
    pub sku: String,

    // #[serde(deserialize_with = "deserialize_number_to_f64")]
    // pub current_price: f64,
    //
    // #[serde(deserialize_with = "deserialize_number_to_f64")]
    // pub regular_price: f64,
    #[serde(deserialize_with = "deserialize_number_to_f64")]
    pub price: f64, // use sqlx::types::Decimal?
    pub category_id: i16,
    pub image_url: String,
    is_available: bool,
}


// Endpoint Callbacks
// Create Product
// curl -X POST http://localhost:8080/api/products \
//   -H "Content-Type: application/json" \
//   -d '{"name": "Iphone", "sku": "iphone1234567tt", "description": "best phone(", "price": 1000, "category_id": 1, "image_url": "https://images/1.webp", "is_available": true}'

pub(crate) async fn create_product(
    data: web::Data<AppState>,
    product_req: web::Json<CreateProductRequest>,
) -> actix_web::Result<HttpResponse> {
    match sqlx::query_as::<_, Product>(
        "INSERT INTO products (name, sku, description, price, category_id, image_url, is_available) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *"
    )
        .bind(&product_req.name)
        .bind(&product_req.sku)
        .bind(&product_req.description)
        .bind(&product_req.price)
        // .bind(&product_req.regular_price)
        .bind(&product_req.category_id)
        .bind(&product_req.image_url)
        .bind(&product_req.is_available)
        .fetch_one(&data.db)
        .await
    {
        Ok(product) => Ok(HttpResponse::Created().json(product)),
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


// curl http://localhost:8080/api/products
pub async fn get_products(data: web::Data<AppState>) -> actix_web::Result<HttpResponse> {

    // SELECT amount::FLOAT8

    // match sqlx::query_as::<_, Product>("SELECT * FROM products ORDER BY created_at DESC")
    match sqlx::query_as::<_, Product>("SELECT product_id, name, sku, description, price::FLOAT8, category_id::INT2, image_url, created_at, updated_at, is_available FROM products ORDER BY created_at DESC")
        .fetch_all(&data.db)
        .await
    {
        Ok(products) => Ok(HttpResponse::Ok().json(products)),
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