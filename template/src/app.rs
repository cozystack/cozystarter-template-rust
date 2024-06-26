use std::sync::Arc;

use axum::{routing::get, Json, Router};
use minijinja::{path_loader, Environment};
use serde_json::json;
use sqlx::{Pool, Postgres};

use crate::handlers::home::home;

pub struct App {
    pub(crate) env: Environment<'static>,
    pub(crate) db: Pool<Postgres>,
}

impl App {
    pub fn new(db: Pool<Postgres>) -> Self {
        let mut env = Environment::new();
        env.set_loader(path_loader("templates"));
        Self { env, db }
    }

    pub fn load_router(self) -> Router {
        let state = Arc::new(self);
        Router::new()
            .route("/health", get(|| async { Json(json!({ "status": "✅"})) }))
            .route("/", get(home))
            .with_state(state)
    }

    pub async fn shutdown_signal() {
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };
        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };
        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();
        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }
    }
}
