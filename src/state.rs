use sqlx::SqlitePool;

/// `AppState` menyimpan seluruh state aplikasi yang dibagikan antar request.
///
/// Konsep Inti Rust di sini:
/// `SqlitePool` secara internal dibungkus oleh `Arc`, sehingga meng-clone `AppState`
/// hanya menambah referensi atomic ke connection pool dan sangat ringan.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
}

impl AppState {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}
