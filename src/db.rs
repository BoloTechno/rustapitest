use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::str::FromStr;

/// Menginisialisasi connection pool SQLite dan menjalankan migrasi schema awal.
pub async fn init_db_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    // Parsing opsi koneksi SQLite dan aktifkan create_if_missing agar file .db otomatis dibuat
    let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);

    // Konfigurasi pool koneksi:
    // Sqlite bekerja paling optimal dengan koneksi terbatas untuk mencegah lock contention
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    // Menjalankan migrasi schema awal (DDL) jika tabel belum ada
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS todos (
            id TEXT PRIMARY KEY NOT NULL,
            title TEXT NOT NULL,
            completed BOOLEAN NOT NULL DEFAULT 0
        );
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
