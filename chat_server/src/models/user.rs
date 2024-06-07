use crate::{AppError, User};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};

impl User {
    pub async fn find_by_email(email: &str, pool: &sqlx::PgPool) -> Result<Option<Self>, AppError> {
        let rec = sqlx::query_as("SELECT id, name, email, created_at FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await?;
        Ok(rec)
    }

    /// Create a new user
    pub async fn create(
        email: &str,
        name: &str,
        password: &str,
        pool: &sqlx::PgPool,
    ) -> Result<Self, AppError> {
        let password_hash = hash_password(password)?;
        let rec = sqlx::query_as(
            r#"
            INSERT INTO users(email,name,password_hash)
            VALUES($1,$2,$3)
            RETURNING id,email,name,password_hash,created_at
            "#,
        )
        .bind(email)
        .bind(name)
        .bind(password_hash)
        .fetch_one(pool)
        .await?;
        Ok(rec)
    }
    /// Verify user's email and password
    pub async fn verify(
        email: &str,
        password: &str,
        pool: &sqlx::PgPool,
    ) -> Result<Option<Self>, AppError> {
        let user: Option<User> = sqlx::query_as(
            "SELECT id, name, email, password_hash, created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;
        match user {
            Some(mut user) => {
                // Verify the password
                let password_hash = user.password_hash.take().unwrap();
                let is_valid = verify_password(password, &password_hash)?;
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
    use sqlx_db_tester::TestPg;
    use std::path::Path;

    #[test]
    fn hash_password_and_verify_should_work() {
        let password = "password";
        let hash = hash_password(password).unwrap();
        assert_eq!(hash.len(), 97);
        assert!(verify_password(password, &hash).unwrap());
    }

    #[tokio::test]
    async fn create_and_verify_user_should_woek() -> anyhow::Result<()> {
        let tdb = TestPg::new(
            "postgres://postgres:postgres@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let (email, name, password) = ("taki@gmail.com", "Taki", "takitaki");
        let user = User::create(email, name, password, &pool).await?;
        assert_eq!(user.email, email);
        assert_eq!(user.name, name);
        assert!(user.id > 0);

        let user = User::find_by_email(email, &pool).await?.unwrap();
        assert_eq!(user.email, email);
        assert_eq!(user.name, name);

        let user = User::verify(email, password, &pool).await?;
        assert!(user.is_some());
        Ok(())
    }
}
