use super::Workspace;
use crate::AppError;
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::warn;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateUser {
    pub name: String,
    pub email: String,
    pub password: String,
    pub workspace: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignInUser {
    pub email: String,
    pub password: String,
}

impl User {
    pub fn new(id: i64, name: impl Into<String>, email: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            email: email.into(),
            password_hash: None,
            created_at: chrono::Utc::now(),
            ws_id: 0,
        }
    }

    pub async fn find_by_email(email: &str, pool: &sqlx::PgPool) -> Result<Option<Self>, AppError> {
        let rec =
            sqlx::query_as("SELECT id, name, email, created_at,ws_id FROM users WHERE email = $1")
                .bind(email)
                .fetch_optional(pool)
                .await?;
        Ok(rec)
    }

    /// Create a new user
    pub async fn create(create_user: &CreateUser, pool: &sqlx::PgPool) -> Result<Self, AppError> {
        let user = User::find_by_email(&create_user.email, pool).await?;
        // Check if the user already exists
        if let Some(user) = user {
            return Err(AppError::AlreadyExists(user.email));
        }
        //find the workspace id
        let ws_id = match Workspace::find_by_name(&create_user.workspace, pool).await? {
            Some(ws) => ws.id,
            None => {
                warn!(
                    "Workspace not found:{}, set default workspace instead",
                    create_user.workspace
                );
                0
            }
        };
        let password_hash = hash_password(&create_user.password)?;
        let rec = sqlx::query_as(
            r#"
            INSERT INTO users(email,name,password_hash,ws_id)
            VALUES($1,$2,$3,$4)
            RETURNING id,email,name,password_hash,created_at,ws_id
            "#,
        )
        .bind(&create_user.email)
        .bind(&create_user.name)
        .bind(password_hash)
        .bind(ws_id)
        .fetch_one(pool)
        .await?;
        Ok(rec)
    }
    /// Verify user's email and password
    pub async fn verify(
        sign_in_user: &SignInUser,
        pool: &sqlx::PgPool,
    ) -> Result<Option<Self>, AppError> {
        let user: Option<User> = sqlx::query_as(
            "SELECT id, name, email, password_hash, created_at,ws_id FROM users WHERE email = $1",
        )
        .bind(&sign_in_user.email)
        .fetch_optional(pool)
        .await?;
        match user {
            Some(mut user) => {
                // Verify the password
                let password_hash = user.password_hash.take().unwrap();
                let is_valid = verify_password(&sign_in_user.password, &password_hash)?;
                if is_valid {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}

impl CreateUser {
    pub fn new(
        workspace: impl Into<String>,
        name: impl Into<String>,
        email: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            email: email.into(),
            name: name.into(),
            password: password.into(),
            workspace: workspace.into(),
        }
    }
}

impl SignInUser {
    pub fn new(email: impl Into<String>, password: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            password: password.into(),
        }
    }
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(password_hash)
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let password_hash = PasswordHash::new(hash)?;
    let is_valid = argon2
        .verify_password(password.as_bytes(), &password_hash)
        .is_ok();
    Ok(is_valid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::get_test_pool;

    #[test]
    fn hash_password_and_verify_should_work() {
        let password = "password";
        let hash = hash_password(password).unwrap();
        assert_eq!(hash.len(), 97);
        assert!(verify_password(password, &hash).unwrap());
    }

    #[tokio::test]
    async fn create_and_verify_user_should_work() -> anyhow::Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;
        let (name, email, password) = ("Taki", "Taki@gmail.com", "takitaki");
        let create_user = CreateUser::new("none", name, email, password);
        let user = User::create(&create_user, &pool).await?;
        assert_eq!(user.email, email);
        assert_eq!(user.name, name);
        assert!(user.id > 0);

        let user = User::find_by_email(email, &pool).await?.unwrap();
        assert_eq!(user.email, email);
        assert_eq!(user.name, name);

        let sign_in_user = SignInUser {
            email: email.to_string(),
            password: password.to_string(),
        };
        let user = User::verify(&sign_in_user, &pool).await?;
        assert!(user.is_some());
        Ok(())
    }
}
