use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

/// Enum yang memodelkan jenis-jenis error yang dapat terjadi di API.
#[derive(Debug)]
pub enum AppError {
    /// Data yang dicari tidak ditemukan (HTTP 404)
    NotFound(String),
    /// Permintaan klien tidak valid / bad input (HTTP 400)
    BadRequest(String),
    /// Terjadi kesalahan query atau koneksi database (HTTP 500)
    Database(String),
}

/// Implementasi `IntoResponse` memungkinkan `AppError` langsung di-return
/// dari handler Axum sebagai respons HTTP yang rapi dan terstruktur.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Database(msg) => {
                tracing::error!("Database error: {msg}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Terjadi kesalahan pada database".to_string(),
                )
            }
        };

        let body = Json(json!({
            "success": false,
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

/// Memungkinkan penggunaan operator `?` pada operasi SQLx untuk otomatis mengubah `sqlx::Error` menjadi `AppError`.
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err.to_string())
    }
}
