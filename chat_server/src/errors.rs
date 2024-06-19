use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputError {
    pub error: String,
}

impl OutputError {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("sql error: {0}")]
    SqlxError(#[from] sqlx::Error),
    #[error("password hash error: {0}")]
    PasswordHashError(#[from] argon2::password_hash::Error),
    #[error("jwt sign error: {0}")]
    JwtSignError(#[from] jwt_simple::Error),
    #[error("user with email {0} already exists")]
    AlreadyExists(String),
    #[error("create chat error: {0}")]
    ChatValidateError(String),
    #[error("not fount error: {0}")]
    NotFountError(String),
    #[error("io error: {0}")]
    IOError(#[from] tokio::io::Error),
    #[error("multipart error: {0}")]
    MultipartError(#[from] axum_extra::extract::multipart::MultipartError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_code = match self {
            AppError::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::PasswordHashError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::JwtSignError(_) => StatusCode::FORBIDDEN,
            AppError::AlreadyExists(_) => StatusCode::CONFLICT,
            AppError::ChatValidateError(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::NotFountError(_) => StatusCode::NOT_FOUND,
            AppError::IOError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::MultipartError(_) => StatusCode::UNPROCESSABLE_ENTITY,
        };
        let body = (status_code, Json(OutputError::new(self.to_string())));
        body.into_response()
    }
}
