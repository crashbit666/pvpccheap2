mod handlers;
mod middleware;
mod models;
mod schema;
mod services;
mod utils;

use actix_cors::Cors;
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, middleware::Logger, web, App, HttpServer};
use deadpool_diesel::postgres::{Manager, Pool};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;
use std::env;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub type DbPool = Pool;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub jwt_secret: String,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub fcm_server_key: String,
    pub encryption_key: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Configuració de la base de dades
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = Manager::new(database_url, deadpool_diesel::Runtime::Tokio1);
    let db_pool = Pool::builder(manager)
        .max_size(8)
        .build()
        .expect("Failed to create database pool");

    // Executar migracions
    {
        let conn = db_pool.get().await.expect("Failed to get DB connection");
        conn.interact(|conn| conn.run_pending_migrations(MIGRATIONS).map(|_| ()))
            .await
            .expect("Failed to interact with connection")
            .expect("Failed to run migrations");
    }

    log::info!("Database migrations completed successfully");

    // Configuració de l'aplicació
    let app_state = AppState {
        db_pool: db_pool.clone(),
        jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
        google_client_id: env::var("GOOGLE_CLIENT_ID")
            .expect("GOOGLE_CLIENT_ID must be set"),
        google_client_secret: env::var("GOOGLE_CLIENT_SECRET")
            .expect("GOOGLE_CLIENT_SECRET must be set"),
        fcm_server_key: env::var("FCM_SERVER_KEY").unwrap_or_default(),
        encryption_key: env::var("ENCRYPTION_KEY").expect("ENCRYPTION_KEY must be set"),
    };

    // Configuració del servidor
    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("Invalid SERVER_PORT");

    let session_key = Key::from(
        env::var("SESSION_KEY")
            .expect("SESSION_KEY must be set")
            .as_bytes(),
    );

    log::info!("Starting server at http://{}:{}", host, port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://localhost:8080")
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
                actix_web::http::header::CONTENT_TYPE,
            ])
            .supports_credentials()
            .max_age(3600);

        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .wrap(cors)
            .wrap(Logger::default())
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                session_key.clone(),
            ))
            .service(web::scope("/api")
                // Auth routes
                .service(web::scope("/auth")
                    .route("/google", web::get().to(handlers::auth::google_login))
                    .route("/google/callback", web::get().to(handlers::auth::google_callback))
                    .route("/logout", web::post().to(handlers::auth::logout))
                    .route("/me", web::get().to(handlers::auth::get_current_user))
                )
                // Mobile sync routes
                .service(web::scope("/mobile")
                    // TODO: Add authentication middleware
                    .route("/sync", web::post().to(handlers::mobile::sync_devices))
                    .route("/heartbeat", web::post().to(handlers::mobile::heartbeat))
                    .route("/command_result", web::post().to(handlers::mobile::command_result))
                )
                // Device routes
                .service(web::scope("/devices")
                    // TODO: Add authentication middleware
                    .route("", web::get().to(handlers::device::list_devices))
                    .route("/{device_id}", web::get().to(handlers::device::get_device))
                    .route("/{device_id}/state", web::get().to(handlers::device::get_device_state))
                    .route("/{device_id}/command", web::post().to(handlers::device::send_command))
                )
                // Rules routes
                .service(web::scope("/rules")
                    // TODO: Add authentication middleware
                    .route("", web::get().to(handlers::rule::list_rules))
                    .route("", web::post().to(handlers::rule::create_rule))
                    .route("/{rule_id}", web::get().to(handlers::rule::get_rule))
                    .route("/{rule_id}", web::put().to(handlers::rule::update_rule))
                    .route("/{rule_id}", web::delete().to(handlers::rule::delete_rule))
                    .route("/preview", web::post().to(handlers::rule::preview_schedule))
                )
                // Schedules routes
                .service(web::scope("/schedules")
                    // TODO: Add authentication middleware
                    .route("", web::get().to(handlers::schedule::list_schedules))
                    .route("/today", web::get().to(handlers::schedule::get_today_schedules))
                    .route("/rebuild", web::post().to(handlers::schedule::rebuild_schedules))
                )
                // WebSocket for real-time updates
                .route("/ws", web::get().to(handlers::websocket::websocket_handler))
            )
            // Health check
            .route("/health", web::get().to(handlers::health::health_check))
    })
    .bind((host, port))?
    .run()
    .await
}