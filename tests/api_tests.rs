use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use rustapitest::{
    create_app, db::init_db_pool, models::ApiResponse, models::Todo, state::AppState,
};
use serde_json::json;
use tower::ServiceExt;
use uuid::Uuid;

async fn setup_test_app() -> axum::Router {
    let pool = init_db_pool("sqlite::memory:")
        .await
        .expect("Gagal inisialisasi in-memory DB");
    create_app(AppState::new(pool))
}

#[tokio::test]
async fn test_health_check() {
    let app = setup_test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let res: ApiResponse<()> = serde_json::from_slice(&body).unwrap();
    assert!(res.success);
    assert_eq!(res.message, "API dan Database berjalan dengan lancar");
}

#[tokio::test]
async fn test_create_and_get_todo() {
    let app = setup_test_app().await;

    // 1. Create Todo
    let create_payload = json!({
        "title": "Mempelajari Axum Framework & SQLx"
    });

    let res_create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(create_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res_create.status(), StatusCode::CREATED);
    let body_bytes = res_create.into_body().collect().await.unwrap().to_bytes();
    let create_result: ApiResponse<Todo> = serde_json::from_slice(&body_bytes).unwrap();
    assert!(create_result.success);

    let created_todo = create_result.data.expect("Data todo harus ada");
    assert_eq!(created_todo.title, "Mempelajari Axum Framework & SQLx");
    assert!(!created_todo.completed);

    // 2. Get Detail Todo
    let res_get = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/todos/{}", created_todo.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res_get.status(), StatusCode::OK);
    let get_bytes = res_get.into_body().collect().await.unwrap().to_bytes();
    let get_result: ApiResponse<Todo> = serde_json::from_slice(&get_bytes).unwrap();
    assert_eq!(get_result.data.unwrap().id, created_todo.id);
}

#[tokio::test]
async fn test_create_todo_validation_empty_title() {
    let app = setup_test_app().await;

    let payload = json!({
        "title": "   "
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_update_and_delete_todo() {
    let app = setup_test_app().await;

    // 1. Buat Todo
    let create_payload = json!({
        "title": "Tugas untuk diperbarui"
    });

    let res_create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(create_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body_bytes = res_create.into_body().collect().await.unwrap().to_bytes();
    let create_result: ApiResponse<Todo> = serde_json::from_slice(&body_bytes).unwrap();
    let todo_id = create_result.data.unwrap().id;

    // 2. Update Todo (Ubah judul dan set completed = true)
    let update_payload = json!({
        "title": "Tugas telah selesai",
        "completed": true
    });

    let res_update = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/todos/{todo_id}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(update_payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res_update.status(), StatusCode::OK);
    let update_bytes = res_update.into_body().collect().await.unwrap().to_bytes();
    let update_result: ApiResponse<Todo> = serde_json::from_slice(&update_bytes).unwrap();
    let updated_todo = update_result.data.unwrap();
    assert_eq!(updated_todo.title, "Tugas telah selesai");
    assert!(updated_todo.completed);

    // 3. Delete Todo
    let res_delete = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/todos/{todo_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res_delete.status(), StatusCode::OK);

    // 4. Pastikan Todo tidak ditemukan lagi (404)
    let res_get_after = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/todos/{todo_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res_get_after.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_not_found() {
    let app = setup_test_app().await;
    let random_id = Uuid::new_v4();

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/todos/{random_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_and_delete_not_found() {
    let app = setup_test_app().await;
    let random_id = Uuid::new_v4();

    // PUT non-existent ID
    let res_put = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/todos/{random_id}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"title": "Update non-existent"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_put.status(), StatusCode::NOT_FOUND);

    // DELETE non-existent ID
    let res_del = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/todos/{random_id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_del.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_empty_title() {
    let app = setup_test_app().await;

    // Create a todo first
    let res_create = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/todos")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"title": "Initial"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let body_bytes = res_create.into_body().collect().await.unwrap().to_bytes();
    let created: ApiResponse<Todo> = serde_json::from_slice(&body_bytes).unwrap();
    let todo_id = created.data.unwrap().id;

    // Update with blank title
    let res_update = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/api/todos/{todo_id}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(json!({"title": "   "}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_update.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_database_persistence_across_connections() {
    // Pengujian khusus: Buktikan bahwa data tetap tersimpan saat connection pool ditutup dan dibuka kembali
    let temp_dir = std::env::temp_dir();
    let db_path = temp_dir.join(format!("test_todo_{}.db", Uuid::new_v4()));
    let db_url = format!("sqlite://{}?mode=rwc", db_path.to_string_lossy());

    let todo_id;
    {
        // Sesi 1: Buat pool, insert data, lalu drop pool (simulasi server shutdown)
        let pool = init_db_pool(&db_url).await.unwrap();
        let app = create_app(AppState::new(pool));

        let res = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/todos")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        json!({"title": "Persistensi database"}).to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::CREATED);
        let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
        let create_result: ApiResponse<Todo> = serde_json::from_slice(&body_bytes).unwrap();
        todo_id = create_result.data.unwrap().id;
    }

    {
        // Sesi 2: Buka pool baru ke file database yang sama (simulasi server restart)
        let pool = init_db_pool(&db_url).await.unwrap();
        let app = create_app(AppState::new(pool));

        let res = app
            .oneshot(
                Request::builder()
                    .uri(format!("/api/todos/{todo_id}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
        let get_result: ApiResponse<Todo> = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(get_result.data.unwrap().title, "Persistensi database");
    }

    // Bersihkan file temporary
    let _ = std::fs::remove_file(db_path);
}
