use crate::{AppError, User};
use jwt_simple::prelude::*;
use std::ops::Deref;

const JWT_DURATION: u64 = 60 * 60 * 24 * 7; // 7 days
const JWT_ISS: &str = "chat_server";
const JWT_AUD: &str = "chat_web";

//generate a jwt token with User struct
pub struct EncodingKey(Ed25519KeyPair);

//verify jwt token
pub struct DecodingKey(Ed25519PublicKey);

impl EncodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        Ok(Self(Ed25519KeyPair::from_pem(pem)?))
    }
    #[allow(unused)]
    pub fn encode(&self, user: User) -> Result<String, AppError> {
        let claims = Claims::with_custom_claims(user, Duration::from_secs(JWT_DURATION));
        let claims = claims.with_audience(JWT_AUD).with_issuer(JWT_ISS);
        let token = self.sign(claims)?;
        Ok(token)
    }
}

impl DecodingKey {
    pub fn load(pem: &str) -> Result<Self, AppError> {
        Ok(Self(Ed25519PublicKey::from_pem(pem)?))
    }

    pub fn verify(&self, token: &str) -> Result<User, AppError> {
        let option = Some(VerificationOptions {
            allowed_issuers: Some(HashSet::from_strings(&[JWT_ISS])),
            allowed_audiences: Some(HashSet::from_strings(&[JWT_AUD])),
            ..Default::default()
        });
        let claims: JWTClaims<User> = self.verify_token(token, option)?;
        Ok(claims.custom)
    }
}

impl Deref for EncodingKey {
    type Target = Ed25519KeyPair;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Deref for DecodingKey {
    type Target = Ed25519PublicKey;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn jwt_sign_verify_should_work() -> anyhow::Result<()> {
        //generate key
        let key_pair = Ed25519KeyPair::generate();
        let encoding_key = EncodingKey(key_pair);

        let decoding_key = DecodingKey(encoding_key.public_key());
        let user = User {
            id: 1,
            name: "test".to_string(),
            password_hash: None,
            email: "taki@gmail.com".to_string(),
            created_at: Utc::now(),
        };
        let token = encoding_key.encode(user.clone())?;
        let decoded_user = decoding_key.verify(&token)?;
        assert_eq!(user, decoded_user);

        //use pem to generate key
        let pem_sk_str = include_str!("../../fixtures/encoding.pem");
        let pem_pk_str = include_str!("../../fixtures/decoding.pem");
        let encoding_key = EncodingKey(Ed25519KeyPair::from_pem(pem_sk_str)?);
        let decoding_key = DecodingKey(Ed25519PublicKey::from_pem(pem_pk_str)?);
        let token = encoding_key.encode(user.clone())?;
        let decoded_user = decoding_key.verify(&token)?;
        assert_eq!(user, decoded_user);

        Ok(())
    }
}
