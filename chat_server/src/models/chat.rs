use crate::{AppError, AppState};
use chat_core::{Chat, ChatType};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};
#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq, ToSchema, IntoParams)]
pub struct CreateChat {
    pub name: Option<String>,
    pub public: bool,
    pub members: Vec<i64>,
}

#[derive(Debug, Clone, FromRow, Serialize, ToSchema, Deserialize, PartialEq, IntoParams)]
pub struct UpdateChat {
    pub name: Option<Option<String>>,
    pub public: Option<bool>,
    pub members: Option<Vec<i64>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ChatUser {
    pub name: String,
    pub email: String,
}

#[allow(unused)]
impl AppState {
    pub async fn create_chat(&self, input: &CreateChat, ws_id: i64) -> Result<Chat, AppError> {
        Self::validate_chat(&input.members, &input.name)?;

        let users = ChatUser::fetch_by_ids(&input.members, &self.pool).await?;
        Self::validate_chat_users(&users, &input.members)?;

        let chat_type = Self::determine_chat_type(input.name.as_ref(), users.len(), input.public);

        let chat = sqlx::query_as(
            r#"
            INSERT INTO chats (ws_id, name, type, members)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, name, type, members, created_at
            "#,
        )
        .bind(ws_id)
        .bind(&input.name)
        .bind(chat_type)
        .bind(&input.members)
        .fetch_one(&self.pool)
        .await?;

        Ok(chat)
    }

    fn validate_chat(members: &[i64], name: &Option<String>) -> Result<(), AppError> {
        if members.len() < 2 {
            return Err(AppError::ChatValidateError(
                "Members should be more than 2".into(),
            ));
        }

        if members.len() > 8 && name.is_none() {
            return Err(AppError::ChatValidateError(
                "Group chat with more than 8 members should have a name".into(),
            ));
        }

        Ok(())
    }

    fn validate_chat_users(users: &[ChatUser], members: &[i64]) -> Result<(), AppError> {
        if users.len() != members.len() {
            return Err(AppError::ChatValidateError(
                "Some members do not exist".into(),
            ));
        }

        Ok(())
    }

    pub async fn update_chat_by_id(&self, input: &UpdateChat, id: i64) -> Result<Chat, AppError> {
        let chat = self
            .fetch_chat_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFountError("Chat not found".into()))?;

        if let Some(members) = &input.members {
            Self::validate_chat(members, &chat.name)?;
            let users = ChatUser::fetch_by_ids(members, &self.pool).await?;
            Self::validate_chat_users(&users, members)?;
        }

        let chat_type = Self::determine_updated_chat_type(input, &chat);

        let updated_chat = sqlx::query_as(
            r#"
            UPDATE chats SET name = $1, type = $2, members = $3
            WHERE id = $4
            RETURNING id, ws_id, name, type, members, created_at
            "#,
        )
        .bind(&input.name)
        .bind(chat_type)
        .bind(&input.members)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(updated_chat)
    }

    fn determine_chat_type(name: Option<&String>, user_count: usize, public: bool) -> ChatType {
        match (name, user_count) {
            (None, 2) => ChatType::Single,
            (None, _) => ChatType::Group,
            (Some(_), _) => {
                if public {
                    ChatType::PublicChannel
                } else {
                    ChatType::PrivateChannel
                }
            }
        }
    }

    fn determine_updated_chat_type(input: &UpdateChat, chat: &Chat) -> ChatType {
        match (&input.members, &input.public) {
            (Some(members), _) => match (input.name.as_ref(), members.len()) {
                (None, 2) => ChatType::Single,
                (None, _) => ChatType::Group,
                (Some(_), _) => {
                    if let Some(public) = input.public {
                        if public {
                            ChatType::PublicChannel
                        } else {
                            ChatType::PrivateChannel
                        }
                    } else {
                        chat.r#type
                    }
                }
            },
            (None, _) => match (input.name.as_ref(), chat.members.len()) {
                (None, 2) => ChatType::Single,
                (None, _) => ChatType::Group,
                (Some(_), _) => {
                    if let Some(public) = input.public {
                        if public {
                            ChatType::PublicChannel
                        } else {
                            ChatType::PrivateChannel
                        }
                    } else {
                        chat.r#type
                    }
                }
            },
        }
    }

    pub async fn fetch_chat_by_id(&self, id: i64) -> Result<Option<Chat>, AppError> {
        let chat = sqlx::query_as::<_, Chat>(
            "SELECT id, ws_id, name, type, members, created_at FROM chats WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(chat)
    }

    pub async fn fetch_chats_by_ws_id(&self, ws_id: i64) -> Result<Vec<Chat>, AppError> {
        let chats = sqlx::query_as::<_, Chat>(
            "SELECT id, ws_id, name, type, members, created_at FROM chats WHERE ws_id = $1",
        )
        .bind(ws_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(chats)
    }

    //TODO: delete chat should also delete messages
    pub async fn delete_chat_by_id(&self, id: i64) -> Result<(), AppError> {
        let res = sqlx::query("DELETE FROM chats WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if res.rows_affected() == 0 {
            return Err(AppError::NotFountError("Chat not found".into()));
        }

        Ok(())
    }

    pub async fn is_chat_member(&self, chat_id: u64, user_id: u64) -> Result<bool, AppError> {
        let is_member = sqlx::query(
            r#"
            SELECT 1
            FROM chats
            WHERE id = $1 AND $2 = ANY(members)
            "#,
        )
        .bind(chat_id as i64)
        .bind(user_id as i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(is_member.is_some())
    }
}

#[allow(unused)]
impl ChatUser {
    pub async fn fetch_by_ids(ids: &[i64], pool: &sqlx::PgPool) -> Result<Vec<Self>, AppError> {
        let users =
            sqlx::query_as::<_, ChatUser>("SELECT id, name, email FROM users WHERE id = ANY($1)")
                .bind(ids)
                .fetch_all(pool)
                .await?;

        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    impl CreateChat {
        pub fn new(name: &str, members: &[i64], public: bool) -> Self {
            let name = if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            };
            Self {
                name,
                members: members.to_vec(),
                public,
            }
        }
    }

    #[tokio::test]
    async fn create_chat_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        //group chat
        let create_chat = CreateChat::new("", &[1, 2, 3], false);
        let chat = state.create_chat(&create_chat, 1).await?;
        assert_eq!(chat.name, None);
        assert_eq!(chat.members, create_chat.members);
        assert_eq!(chat.r#type, ChatType::Group);

        //single chat
        let create_chat = CreateChat::new("", &[1, 2], false);
        let chat = state.create_chat(&create_chat, 1).await?;
        assert_eq!(chat.name, None);
        assert_eq!(chat.members, create_chat.members);
        assert_eq!(chat.r#type, ChatType::Single);

        //private channel
        let create_chat = CreateChat::new("test chat", &[1, 2, 3], false);
        let chat = state.create_chat(&create_chat, 1).await?;
        assert_eq!(chat.name, create_chat.name);
        assert_eq!(chat.members, create_chat.members);
        assert_eq!(chat.r#type, ChatType::PrivateChannel);

        //public channel
        let create_chat = CreateChat::new("test chat", &[1, 2, 3], true);
        let chat = state.create_chat(&create_chat, 1).await?;
        assert_eq!(chat.name, create_chat.name);
        assert_eq!(chat.members, create_chat.members);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        Ok(())
    }

    #[tokio::test]
    async fn fetch_chat_by_id_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let fetched_chat = state.fetch_chat_by_id(1).await?.unwrap();
        assert_eq!(fetched_chat.id, 1);
        assert_eq!(fetched_chat.name, Some("single_chat".into()));
        assert_eq!(fetched_chat.r#type, ChatType::Single);
        assert_eq!(fetched_chat.members, vec![1, 2]);
        Ok(())
    }

    #[tokio::test]
    async fn fetch_all_chats_by_ws_id_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let fetched_chats = state.fetch_chats_by_ws_id(0).await?;
        assert_eq!(fetched_chats.len(), 6);
        let chat = &fetched_chats[0];
        assert_eq!(chat.id, 1);
        assert_eq!(chat.name, Some("single_chat".into()));
        assert_eq!(chat.r#type, ChatType::Single);
        assert_eq!(chat.members, vec![1, 2]);
        Ok(())
    }

    #[tokio::test]
    async fn update_chat_by_id_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let update_chat = UpdateChat {
            name: Some(Some("new_name".into())),
            public: Some(true),
            members: Some(vec![1, 2, 3]),
        };
        let chat = state.update_chat_by_id(&update_chat, 1).await?;
        assert_eq!(chat.name, update_chat.name.unwrap());
        assert_eq!(chat.members, update_chat.members.unwrap());
        assert_eq!(chat.r#type, ChatType::PublicChannel);

        let update_chat = UpdateChat {
            name: None,
            public: Some(true),
            members: Some(vec![1, 2]),
        };
        let chat = state.update_chat_by_id(&update_chat, 1).await?;
        assert_eq!(chat.name, None);
        assert_eq!(chat.members, update_chat.members.unwrap());
        assert_eq!(chat.r#type, ChatType::Single);

        Ok(())
    }

    #[tokio::test]
    async fn chat_is_member_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let is_member = state.is_chat_member(5, 1).await.expect("is member failed");
        assert!(is_member);

        // user 6 doesn't exist
        let is_member = state.is_chat_member(1, 6).await.expect("is member failed");
        assert!(!is_member);

        // chat 10 doesn't exist
        let is_member = state.is_chat_member(10, 1).await.expect("is member failed");
        assert!(!is_member);

        // user 4 is not a member of chat 1
        let is_member = state.is_chat_member(1, 4).await.expect("is member failed");
        assert!(!is_member);

        Ok(())
    }

    #[tokio::test]
    async fn delete_chat_by_id_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chat_id = 1;
        let fetched_chat = state.fetch_chat_by_id(chat_id).await?;
        assert!(fetched_chat.is_some());
        state.delete_chat_by_id(chat_id).await?;
        let fetched_chat = state.fetch_chat_by_id(chat_id).await?;
        assert!(fetched_chat.is_none());
        Ok(())
    }
}
