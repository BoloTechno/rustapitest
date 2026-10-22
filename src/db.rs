use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions},
};
use std::{str::FromStr, time::Duration};

/// Menginisialisasi connection pool SQLite dan menjalankan migrasi schema awal.
pub async fn init_db_pool(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    // Parsing opsi koneksi SQLite: aktifkan create_if_missing, WAL mode, dan timeout 5 detik
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));

    // Konfigurasi pool koneksi
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
