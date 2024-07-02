use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Extension, Json,
};
use axum_extra::extract::Multipart;
use std::vec;
use tokio::fs;
use tracing::{info, warn};

use crate::{models::ChatFile, AppError, AppState, CreateMessage, ListMessages};
use chat_core::User;

#[utoipa::path(
    get,
    path = "/api/chats/{id}/messages",
    params(
        ("id" = u64, Path, description = "Chat id"),
        ListMessages
    ),
    responses(
        (status = 200, description = "List of messages", body = Vec<Message>),
        (status = 400, description = "Invalid input", body = OutputError),
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn list_message_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Query(input): Query<ListMessages>,
) -> Result<impl IntoResponse, AppError> {
    let messages = state.list_messages(input, id).await?;
    Ok(Json(messages))
}

#[utoipa::path(
    post,
    path = "/api/chats/{id}",
    params(
        ("id" = u64, Path, description = "Chat id"),
        CreateMessage
    ),
    responses(
        (status = 201, description = "Send a message", body = Message),
    ),
    security(
        ("token" = [])
    )
)]
pub(crate) async fn send_message_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(input): Json<CreateMessage>,
) -> Result<impl IntoResponse, AppError> {
    let msg = state.create_message(input, id, user.id as _).await?;

    Ok((StatusCode::CREATED, Json(msg)))
}

///handle file upload
pub async fn file_upload_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let ws_id = user.ws_id as u64;
    let path = &state.config.server.base_url;
    let mut files = vec![];
    while let Some(field) = multipart.next_field().await? {
        let name = field.file_name().map(|name| name.to_string());
        let (filename, bytes) = match (name, field.bytes().await) {
            (Some(filename), Ok(bytes)) => (filename, bytes),
            (a, b) => {
                warn!("file name or bytes not found");
                warn!("file name: {:?},file bytes: {:?}", a, b);
                continue;
            }
        };
        let chat_file = ChatFile::new(ws_id, &filename, &bytes);
        let path = chat_file.path(path);
        if !path.exists() {
            fs::create_dir_all(path.parent().expect("file path parent should exists")).await?;
            fs::write(&path, &bytes).await?;
        } else {
            warn!("file already exists");
        }
        files.push(chat_file.url());
    }
    Ok((StatusCode::OK, Json(files)))
}

///handle file download
pub async fn file_download_handler(
    Path((ws_id, path)): Path<(i64, String)>,
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    if ws_id != user.ws_id {
        return Err(AppError::NotFountError(
            "file not found or you don't have permission".to_string(),
        ));
    }
    let path = state
        .config
        .server
        .base_url
        .join(ws_id.to_string())
        .join(path);
    if !path.exists() {
        return Err(AppError::NotFountError("file not found".to_string()));
    }
    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    info!("file mime type: {}", mime);
    let mut header = HeaderMap::new();
    let body = fs::read(path).await?;
    header.insert("Content-Type", mime.to_string().parse().unwrap());
    Ok((header, body))
}
