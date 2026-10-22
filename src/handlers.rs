use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{ApiResponse, CreateTodoPayload, Todo, UpdateTodoPayload},
    state::AppState,
};

/// GET /health
/// Endpoint untuk memeriksa apakah server berjalan dan database dapat diakses.
pub async fn health_check(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<ApiResponse<()>>), AppError> {
    // Ping database untuk memastikan koneksi aktif
    sqlx::query("SELECT 1").execute(&state.pool).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::ok_msg(
            "API dan Database berjalan dengan lancar",
        )),
    ))
}

/// GET /api/todos
/// Mengambil daftar semua todo yang tersimpan di database SQLite.
pub async fn list_todos(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Todo>>>, AppError> {
    let todos = sqlx::query_as::<_, Todo>(
        "SELECT id, title, completed FROM todos ORDER BY rowid ASC LIMIT 100",
    )
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(ApiResponse::success(
        "Daftar todo berhasil diambil",
        todos,
    )))
}

/// GET /api/todos/:id
/// Mengambil detail satu todo berdasarkan UUID dari database.
pub async fn get_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Todo>>, AppError> {
    let maybe_todo =
        sqlx::query_as::<_, Todo>("SELECT id, title, completed FROM todos WHERE id = ?")
            .bind(id)
            .fetch_optional(&state.pool)
            .await?;

    match maybe_todo {
        Some(todo) => Ok(Json(ApiResponse::success("Detail todo ditemukan", todo))),
        None => Err(AppError::NotFound(format!(
            "Todo dengan ID '{id}' tidak ditemukan"
        ))),
    }
}

/// POST /api/todos
/// Menyimpan todo baru ke dalam tabel SQLite.
pub async fn create_todo(
    State(state): State<AppState>,
    Json(payload): Json<CreateTodoPayload>,
) -> Result<(StatusCode, Json<ApiResponse<Todo>>), AppError> {
    let title = payload.title.trim().to_string();

    // Validasi sederhana: judul tidak boleh kosong
    if title.is_empty() {
        return Err(AppError::BadRequest(
            "Judul (title) tidak boleh kosong".to_string(),
        ));
    }

    let new_todo = Todo {
        id: Uuid::new_v4(),
        title,
        completed: false,
    };

    sqlx::query("INSERT INTO todos (id, title, completed) VALUES (?, ?, ?)")
        .bind(new_todo.id)
        .bind(&new_todo.title)
        .bind(new_todo.completed)
        .execute(&state.pool)
        .await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success("Todo berhasil dibuat", new_todo)),
    ))
}

/// PUT /api/todos/:id
/// Memperbarui judul dan/atau status selesai (completed) pada database SQLite.
pub async fn update_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTodoPayload>,
) -> Result<Json<ApiResponse<Todo>>, AppError> {
    let trimmed_title = match payload.title {
        Some(t) => {
            let s = t.trim().to_string();
            if s.is_empty() {
                return Err(AppError::BadRequest(
                    "Judul baru tidak boleh kosong".to_string(),
                ));
            }
            Some(s)
        }
        None => None,
    };

    let maybe_todo = sqlx::query_as::<_, Todo>(
        "UPDATE todos SET title = COALESCE(?, title), completed = COALESCE(?, completed) WHERE id = ? RETURNING id, title, completed",
    )
    .bind(trimmed_title)
    .bind(payload.completed)
    .bind(id)
    .fetch_optional(&state.pool)
    .await?;

    match maybe_todo {
        Some(todo) => Ok(Json(ApiResponse::success("Todo berhasil diperbarui", todo))),
        None => Err(AppError::NotFound(format!(
            "Todo dengan ID '{id}' tidak ditemukan"
        ))),
    }
}

/// DELETE /api/todos/:id
/// Menghapus todo dari database SQLite berdasarkan UUID.
pub async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    let result = sqlx::query("DELETE FROM todos WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound(format!(
            "Todo dengan ID '{id}' tidak ditemukan"
        )));
    }

    Ok(Json(ApiResponse::ok_msg("Todo berhasil dihapus")))
}
