use rustapitest::{create_app, db::init_db_pool, state::AppState};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inisialisasi format logging (tracing) di terminal
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rustapitest=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Mengambil koneksi database dari env atau default ke file lokal todos.db
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://todos.db?mode=rwc".to_string());

    tracing::info!("Menghubungkan ke database: {}", database_url);
    let pool = init_db_pool(&database_url).await?;

    // Inisialisasi State aplikasi dengan connection pool
    let state = AppState::new(pool);

    // Bangun router dengan state
    let app = create_app(state);

    // Konfigurasi IP dan port listener dari env atau default ke 0.0.0.0:3000
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3000);
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    tracing::info!("Server API berjalan di http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
