use crate::{AppError, User};

use super::Workspace;

impl Workspace {
    ///create a new workspace
    #[allow(unused)]
    pub async fn create(name: &str, owner_id: i64, pool: &sqlx::PgPool) -> Result<Self, AppError> {
        let rec = sqlx::query_as(
            r#"
            INSERT INTO workspaces(name, owner_id)
            VALUES($1,$2)
            RETURNING id, name, owner_id, created_at
            "#,
        )
        .bind(name)
        .bind(owner_id)
        .fetch_one(pool)
        .await?;
        Ok(rec)
    }

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

    /// fetch_all_chat_users
    #[allow(unused)]
    pub async fn fetch_all_chat_users(
        ws_id: i64,
        pool: &sqlx::PgPool,
    ) -> Result<Vec<User>, AppError> {
        let recs =
            sqlx::query_as("SELECT id, name, email, created_at, ws_id FROM users WHERE ws_id = $1")
                .bind(ws_id)
                .fetch_all(pool)
                .await?;
        Ok(recs)
    }

    ///find workspace by name
    pub async fn find_by_name(name: &str, pool: &sqlx::PgPool) -> Result<Option<Self>, AppError> {
        let rec =
            sqlx::query_as("SELECT id, name, owner_id, created_at FROM workspaces WHERE name = $1")
                .bind(name)
                .fetch_optional(pool)
                .await?;
        Ok(rec)
    }

    ///find workspace by id
    #[allow(unused)]
    pub async fn find_by_id(id: i64, pool: &sqlx::PgPool) -> Result<Option<Self>, AppError> {
        let rec =
            sqlx::query_as("SELECT id, name, owner_id, created_at FROM workspaces WHERE id = $1")
                .bind(id)
                .fetch_optional(pool)
                .await?;
        Ok(rec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::CreateUser, tests::get_test_pool, User};
    use anyhow::{Ok, Result};

    #[tokio::test]
    async fn workspace_create_and_set_owner_should_work() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;
        let ws = Workspace::create("test", 0, &pool).await.unwrap();
        let input = CreateUser::new(&ws.name, "Taki", "Taki@gmail.com", "takitaki");
        let user = User::create(&input, &pool).await.unwrap();
        assert_eq!(ws.name, "test");
        assert_eq!(user.ws_id, ws.id);
        let ws = ws.change_owner(user.id, &pool).await.unwrap();
        assert_eq!(ws.owner_id, user.id);
        Ok(())
    }

    #[tokio::test]
    async fn workspace_find_by_name_should_work() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;
        let ws = Workspace::find_by_name("ws1", &pool).await?;
        //workspace exists
        assert_eq!(ws.unwrap().name, "ws1");
        //workspace does not exist
        let ws = Workspace::find_by_name("ws-1", &pool).await?;
        assert_eq!(ws, None);
        Ok(())
    }

    #[tokio::test]
    async fn workspace_find_by_ids_should_work() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;
        let ws = Workspace::find_by_id(1, &pool).await?;
        //workspace exists
        assert_eq!(ws.unwrap().id, 1);
        //workspace does not exist
        let ws = Workspace::find_by_id(-1, &pool).await?;
        assert_eq!(ws, None);
        Ok(())
    }

    #[tokio::test]
    async fn workspace_fetch_all_chat_users_should_work() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;

        let users = Workspace::fetch_all_chat_users(1, &pool).await?;
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].name, "taki");
        assert_eq!(users[1].name, "okudera");

        assert_eq!(users[0].ws_id, 1);
        assert_eq!(users[1].ws_id, 1);

        Ok(())
    }
}
