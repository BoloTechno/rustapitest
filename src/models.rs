use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Representasi data Todo utama.
/// `Serialize` memungkinkan struct ini diubah menjadi format JSON saat dikirim ke klien.
/// `Deserialize` memungkinkan pembuatan struct dari data JSON.
/// `Clone` memungkinkan duplikasi data saat diambil dari state memori.
/// `sqlx::FromRow` memungkinkan mapping otomatis dari baris tabel database ke struct ini.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, sqlx::FromRow)]
pub struct Todo {
    pub id: Uuid,
    pub title: String,
    pub completed: bool,
}

/// DTO (Data Transfer Object) untuk request pembuatan Todo baru (POST /api/todos).
#[derive(Debug, Deserialize)]
pub struct CreateTodoPayload {
    pub title: String,
}

/// DTO untuk request pembaruan Todo (PUT /api/todos/:id).
/// Menggunakan `Option` sehingga klien bisa memperbarui title saja, status completed saja, atau keduanya.
#[derive(Debug, Deserialize)]
pub struct UpdateTodoPayload {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

/// Format standar respons API untuk konsistensi keluaran.
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(message: impl Into<String>, data: T) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
        }
    }
}

impl ApiResponse<()> {
    pub fn ok_msg(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
        }
    }
}
