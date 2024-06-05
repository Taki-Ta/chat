mod config;
mod handlers;

pub use config::AppConfig;
use handlers::*;
use axum::{routing::{get, patch, post}, Router};
use std::{ops::Deref, sync::Arc};

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    pub(crate) inner: Arc<AppStateInner>,
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
}

impl Deref for AppState {
    type Target = AppStateInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            inner: Arc::new(AppStateInner { config }),
        }
    }
}

pub fn get_router(config: AppConfig) -> Router {
    let state = AppState::new(config);

    let api = Router::new()
        .route("/signin", post(login_handler))
        .route("/signup", post(register_handler))
        .route("/chat", get(list_chat_handler).post(create_chat_handler))
        .route("/chat/:id",patch(update_chat_handler).delete(delete_chat_handler))
        .route("/chat/:id/messages",get(list_messages_handler).post(send_message_handler))
        
        ;
    Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state)
}
