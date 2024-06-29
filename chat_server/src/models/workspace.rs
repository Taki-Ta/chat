use crate::{AppError, AppState, User};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
    pub owner_id: i64,
    pub created_at: DateTime<Utc>,
}

impl AppState {
    ///create a new workspace
    #[allow(unused)]
    pub async fn create_workspace(&self, name: &str, owner_id: i64) -> Result<Workspace, AppError> {
        let rec = sqlx::query_as(
            r#"
            INSERT INTO workspaces(name, owner_id)
            VALUES($1,$2)
            RETURNING id, name, owner_id, created_at
            "#,
        )
        .bind(name)
        .bind(owner_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(rec)
    }

    /// fetch_all_chat_users
    #[allow(unused)]
    pub async fn fetch_workspace_all_chat_users(&self, ws_id: i64) -> Result<Vec<User>, AppError> {
        let recs =
            sqlx::query_as("SELECT id, name, email, created_at, ws_id FROM users WHERE ws_id = $1")
                .bind(ws_id)
                .fetch_all(&self.pool)
                .await?;
        Ok(recs)
    }

    ///find workspace by name
    pub async fn find_workspace_by_name(&self, name: &str) -> Result<Option<Workspace>, AppError> {
        let rec =
            sqlx::query_as("SELECT id, name, owner_id, created_at FROM workspaces WHERE name = $1")
                .bind(name)
                .fetch_optional(&self.pool)
                .await?;
        Ok(rec)
    }

    ///find workspace by id
    #[allow(unused)]
    pub async fn find_workspace_by_id(&self, id: i64) -> Result<Option<Workspace>, AppError> {
        let rec =
            sqlx::query_as("SELECT id, name, owner_id, created_at FROM workspaces WHERE id = $1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await?;
        Ok(rec)
    }
}

impl Workspace {
    ///change the owner of the workspace if the user is the owner
    #[allow(unused)]
    pub async fn change_owner(
        &self,
        new_owner_id: i64,
        pool: &sqlx::PgPool,
    ) -> Result<Self, AppError> {
        let ws=sqlx::query_as("UPDATE workspaces SET owner_id = $1 WHERE id = $2 and owner_id = $3 RETURNING id, name, owner_id, created_at")
            .bind(new_owner_id)
            .bind(self.id)
            .bind(self.owner_id)
            .fetch_one(pool)
            .await?;
        Ok(ws)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::models::CreateUser;
    use anyhow::{Ok, Result};

    #[tokio::test]
    async fn workspace_create_and_set_owner_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws = state.create_workspace("test", 0).await.unwrap();
        let input = CreateUser::new(&ws.name, "Taki", "Taki@gmail.com", "takitaki");
        let user = state.create_user(&input).await.unwrap();
        assert_eq!(ws.name, "test");
        assert_eq!(user.ws_id, ws.id);
        let ws = ws.change_owner(user.id, &state.pool).await.unwrap();
        assert_eq!(ws.owner_id, user.id);
        Ok(())
    }

    #[tokio::test]
    async fn workspace_find_by_name_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws = state.find_workspace_by_name("ws1").await?;
        //workspace exists
        assert_eq!(ws.unwrap().name, "ws1");
        //workspace does not exist
        let ws = state.find_workspace_by_name("ws-1").await?;
        assert_eq!(ws, None);
        Ok(())
    }

    #[tokio::test]
    async fn workspace_find_by_ids_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws = state.find_workspace_by_id(1).await?;
        //workspace exists
        assert_eq!(ws.unwrap().id, 1);
        //workspace does not exist
        let ws = state.find_workspace_by_id(-1).await?;
        assert_eq!(ws, None);
        Ok(())
    }

    #[tokio::test]
    async fn workspace_fetch_all_chat_users_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let users = state.fetch_workspace_all_chat_users(1).await?;
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].name, "taki");
        assert_eq!(users[1].name, "okudera");

        assert_eq!(users[0].ws_id, 1);
        assert_eq!(users[1].ws_id, 1);

        Ok(())
    }
}
