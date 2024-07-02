pub mod middlewares;
mod utils;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

pub use utils::*;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct Chat {
    pub id: i64,
    pub ws_id: i64,
    pub name: Option<String>,
    pub r#type: ChatType,
    pub members: Vec<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug, sqlx::Type, ToSchema)]
#[sqlx(type_name = "chat_type", rename_all = "snake_case")]
pub enum ChatType {
    Single,
    Group,
    PrivateChannel,
    PublicChannel,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct Message {
    pub id: i64,
    pub chat_id: i64,
    pub sender_id: i64,
    pub content: String,
    pub files: Vec<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct User {
    pub id: i64,
    // pub ws_id: String,
    pub name: String,
    #[sqlx(default)]
    #[serde(skip)]
    pub password_hash: Option<String>,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub ws_id: i64,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
    pub owner_id: i64,
    pub created_at: DateTime<Utc>,
}
