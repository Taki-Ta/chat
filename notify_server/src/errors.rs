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
    #[error("jwt sign error: {0}")]
    JwtSignError(#[from] jwt_simple::Error),
    #[error("io error: {0}")]
    IOError(#[from] tokio::io::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_code = match self {
            AppError::JwtSignError(_) => StatusCode::FORBIDDEN,
            AppError::IOError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let body = (status_code, Json(OutputError::new(self.to_string())));
        body.into_response()
    }
}
