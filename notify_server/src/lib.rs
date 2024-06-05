mod sse;

use axum::{
    response::{Html, IntoResponse}, routing::get, Router
};
use sse::sse_handler;

const INDEX_HTML:&'static str=include_str!("../index.html");

pub fn get_router() -> Router {
    Router::new().route("/events", get(sse_handler))
    .route("/index", get(index_handler))
}

pub(crate) async fn index_handler()->impl IntoResponse{
    Html(INDEX_HTML)
}
