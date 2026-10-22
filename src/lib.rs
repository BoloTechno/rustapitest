pub mod db;
pub mod error;
pub mod handlers;
pub mod models;
pub mod state;

use axum::{
    Router,
    routing::{delete, get, post, put},
};
use state::AppState;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};

/// Membuat dan mengonfigurasi Axum Router aplikasi.
/// Memisahkan fungsi ini ke lib.rs memudahkan pengujian otomatis (automated testing).
pub fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(handlers::health_check))
        .route("/api/todos", get(handlers::list_todos))
        .route("/api/todos", post(handlers::create_todo))
        .route("/api/todos/{id}", get(handlers::get_todo))
        .route("/api/todos/{id}", put(handlers::update_todo))
        .route("/api/todos/{id}", delete(handlers::delete_todo))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}
