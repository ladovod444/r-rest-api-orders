//https://github.com/ladovod444/r-rest-api-orders

// src/main.rs
use actix_web::{web, App, HttpResponse, HttpServer, Result};
// use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use uuid::Uuid;


use std::env;

mod user;
mod product;
mod order;
mod order_items;
// pub use user::User;
// pub use user::CreateUserRequest;
// pub use user::UpdateUserRequest;
// use crate::product::CreateProductRequest;

// pub use user::create_user;
// pub use user::update_user;
// pub use user::get_user;
// pub use user::get_users;
// pub use user::delete_user;

use user::*;


// use product::Product;
use product::create_product;
use product::get_products;

use order::*;
use crate::order_items::{create_order_item, get_order_items};

// App state
struct AppState {
    db: Pool<Postgres>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // let default_value = "default_val".to_string();
    // let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| default_value);

    let default_value = "DATABASE_URL=postgres://db:db@localhost:5432/shop-rust".to_string();
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| default_value);
    //     .expect("DATABASE_URL must be set in .env file");

    // let mut database_url

    // let mut database_url = "DATABASE_URL";
    // database_url = match env::var(&database_url) {
    //     Ok(database_url) => &database_url,
    //     Err(e) => "couldn't interpret {key}: {e}",
    // };

    // Create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    // Run migrations
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            user_id SERIAL PRIMARY KEY,
            username VARCHAR(50) UNIQUE NOT NULL,
            email VARCHAR(100) UNIQUE NOT NULL,
            password_hash VARCHAR(255) NOT NULL,
            first_name VARCHAR(50),
            last_name VARCHAR(50),
            phone VARCHAR(20),
            address TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            is_active BOOLEAN DEFAULT TRUE
        );
        "#
    )
        .execute(&pool)
        .await
        .expect("Failed to create users table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS products (
            product_id SERIAL PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            description TEXT,
            sku VARCHAR(50) UNIQUE,
            price DECIMAL(10, 2) NOT NULL CHECK (price >= 0),
            stock_quantity INTEGER NOT NULL DEFAULT 0 CHECK (stock_quantity >= 0),
            category VARCHAR(50),
            image_url VARCHAR(255),
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            is_available BOOLEAN DEFAULT TRUE
        );
        "#
    )
        .execute(&pool)
        .await
        .expect("Failed to create products table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS orders (
            order_id SERIAL PRIMARY KEY,
            user_id INTEGER NOT NULL,
            order_number VARCHAR(50) UNIQUE NOT NULL,
            order_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            total_amount DECIMAL(10, 2) NOT NULL CHECK (total_amount >= 0),
            status VARCHAR(20) DEFAULT 'pending'
                CHECK (status IN ('pending', 'processing', 'shipped', 'delivered', 'cancelled')),
            shipping_address TEXT NOT NULL,
            billing_address TEXT,
            payment_method VARCHAR(30),
            payment_status VARCHAR(20) DEFAULT 'unpaid'
                CHECK (payment_status IN ('unpaid', 'paid', 'refunded', 'failed')),
            notes TEXT,
            FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
        );
        "#
    )
        .execute(&pool)
        .await
        .expect("Failed to create orders table");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS order_items (
            order_item_id SERIAL PRIMARY KEY,
            order_id INTEGER NOT NULL,
            product_id INTEGER NOT NULL,
            quantity INTEGER NOT NULL CHECK (quantity > 0),
            unit_price DECIMAL(10, 2) NOT NULL CHECK (unit_price >= 0),
            subtotal DECIMAL(10, 2) GENERATED ALWAYS AS (quantity * unit_price) STORED,
            FOREIGN KEY (order_id) REFERENCES orders(order_id) ON DELETE CASCADE,
            FOREIGN KEY (product_id) REFERENCES products(product_id) ON DELETE RESTRICT,
            UNIQUE (order_id, product_id) -- предотвращает дублирование одного продукта в заказе
        );
        "#
    )
        .execute(&pool)
        .await
        .expect("Failed to create order_items table");

    let app_state = web::Data::new(AppState { db: pool });

    println!("🚀 Server running at http://localhost:8080");
    println!("📊 Database connected: {}", database_url);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(
                web::scope("/api")
                    .route("/health", web::get().to(health_check))
                    .route("/users", web::get().to(get_users))
                    .route("/users", web::post().to(create_user))
                    .route("/users/{id}", web::get().to(get_user))
                    .route("/users/{id}", web::put().to(update_user))
                    .route("/users/{id}", web::delete().to(delete_user))


                    // TODO доделать остальные методы

                    .route("/products", web::post().to(create_product))
                    .route("/products", web::get().to(get_products))

                    .route("/orders", web::post().to(create_order))
                    .route("/orders", web::get().to(get_orders))

                    .route("/order-items", web::post().to(create_order_item))
                    .route("/order-items", web::get().to(get_order_items)),
            )
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

// Health check with database connection test
// curl http://localhost:8080/api/health
async fn health_check(data: web::Data<AppState>) -> Result<HttpResponse> {
    // Test database connection
    match sqlx::query("SELECT 1").execute(&data.db).await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "status": "ok",
            "database": "connected"
        }))),
        Err(e) => Ok(HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "error",
            "database": "disconnected",
            "error": e.to_string()
        }))),
    }
}
