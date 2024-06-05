mod chat;
mod auth;
mod messages;

use axum::response::IntoResponse;
pub(crate) use chat::*;
pub(crate) use auth::*;
pub(crate) use messages::*;


pub(crate) async fn index_handler()->impl IntoResponse{
    "index_handler"
}