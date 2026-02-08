use std::str::FromStr;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use crate::AppState;
use crate::order::Order;
use crate::product::{CreateProductRequest, Product};
use crate::user::User;

// Data models
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrderItem {
    // id: Uuid, // TODO
    // Order_id: i64, // TODO
    // Order_id: i64, // TODO

    #[sqlx(try_from = "i32")]
    pub order_item_id: i64,

    #[sqlx(try_from = "i32")]
    pub order_id: i64,

    #[sqlx(try_from = "i32")]
    pub product_id: i64,

    #[sqlx(try_from = "i32")]
    pub quantity: i64,

    // pub price: Decimal,

    /* // TODO
    pub unit_price: f64, // use sqlx::types::Decimal?

    pub subtotal: f64, // use sqlx::types::Decimal?
    */




}

// TODO Requests ...

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateOrderItemRequest {
    // pub user_id: i64,

    // #[sqlx(try_from = "i32")]
    pub order_id: i64,

    // #[sqlx(try_from = "i32")]
    pub product_id: i64,

    // #[sqlx(try_from = "i32")]
    pub quantity: i64,

    // pub price: Decimal,

    pub unit_price: f64, // use sqlx::types::Decimal?

    // pub subtotal: f64, // use sqlx::types::Decimal?
}

// Endpoint Callbacks
// Create Order Item
// curl -X POST http://localhost:8080/api/order-items \
// -H "Content-Type: application/json" \
// -d '{"order_id": 7, "product_id": 4, "quantity": 10, "unit_price": 1000}'

pub(crate) async fn create_order_item(
    data: web::Data<AppState>,
    order_item_req: web::Json<CreateOrderItemRequest>,
) -> actix_web::Result<HttpResponse> {
    match sqlx::query_as::<_, OrderItem>(
        // "INSERT INTO order_items (order_id, product_id, quantity, unit_price, subtotal) VALUES ($1, $2, $3, $4, $5) RETURNING *"
        "INSERT INTO order_items (order_id, product_id, quantity, unit_price) VALUES ($1, $2, $3, $4) RETURNING *"
    )
        .bind(&order_item_req.order_id)
        .bind(&order_item_req.product_id)
        .bind(&order_item_req.quantity)
        .bind(&order_item_req.unit_price)
        // .bind(&order_item_req.subtotal)

        .fetch_one(&data.db)
        .await
    {
        Ok(order_item) => Ok(HttpResponse::Created().json(order_item)),
        Err(e) => {
            // Handle unique constraint violation
            if e.to_string().contains("unique constraint") {
                Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Item is already exists"
                })))
            } else {
                Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": e.to_string()
                })))
            }
        }
    }
}

// curl http://localhost:8080/api/order-items
pub async fn get_order_items(data: web::Data<AppState>) -> actix_web::Result<HttpResponse> {

    // SELECT amount::FLOAT8

    // match sqlx::query_as::<_, Order>("SELECT * FROM orders ORDER BY created_at DESC")
    match sqlx::query_as::<_, Order>("SELECT order_item_id, order_id, product_id, quantity, unit_price::FLOAT8, subtotal::FLOAT8 FROM order_items ORDER BY order_id DESC")
        .fetch_all(&data.db)
        .await
    {
        Ok(order_items) => Ok(HttpResponse::Ok().json(order_items)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": e.to_string()
        }))),
    }
}