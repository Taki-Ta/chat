use crate::{
    models::{CreateChat, UpdateChat},
    AppError, AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use chat_core::User;

#[utoipa::path(
    get,
    path = "/api/chats",
    responses(
        (status = 200, description = "List of chats", body = Vec<Chat>),
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chats = state.fetch_chats_by_ws_id(user.ws_id).await?;
    Ok((StatusCode::OK, Json(chats)))
}

#[utoipa::path(
    post,
    path = "/api/chats",
    responses(
        (status = 201, description = "Create chat", body = Chat),
    ),
    params(
        ("id" = u64, Path, description = "Chat id"),
        CreateChat
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.create_chat(&input, user.ws_id).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}

#[utoipa::path(
    patch,
    path = "/api/chats/{id}",
    responses(
        (status = 200, description = "Update chat", body = Chat),
        (status = 404, description = "Chat not found", body = OutputError),
    ),
    params(
        ("id" = u64, Path, description = "Chat id"),
        UpdateChat
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn update_chat_handler(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(input): Json<UpdateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = state.update_chat_by_id(&input, id).await?;
    Ok((StatusCode::OK, Json(chat)))
}

#[utoipa::path(
    delete,
    path = "/api/chats/{id}",
    responses(
        (status = 204, description = "delete chat"),
        (status = 404, description = "Chat not found", body = OutputError),
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn delete_chat_handler(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    state.delete_chat_by_id(id).await?;
    Ok(StatusCode::NO_CONTENT)
}
