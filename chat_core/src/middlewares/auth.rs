use crate::middlewares::TokenVerifier;
use axum::{
    extract::{FromRequestParts, Query, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde::Deserialize;
use tracing::warn;

#[derive(Debug, Deserialize)]
struct Params {
    access_token: String,
}

pub async fn verify_token<T>(State(state): State<T>, mut req: Request, next: Next) -> Response
where
    T: TokenVerifier + Clone + Send + Sync + 'static,
{
    //get token from the request headers and validate it
    let (mut parts, body) = req.into_parts();
    let token =
        match TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await {
            Ok(TypedHeader(Authorization(bearer))) => bearer.token().to_owned(),
            Err(e) => {
                if e.is_missing() {
                    match Query::<Params>::from_request_parts(&mut parts, &state).await {
                        Ok(Query(params)) => params.access_token,
                        Err(e) => {
                            let msg = format!("parse Authorization header failed: {}", e);
                            warn!(msg);
                            return (
                                StatusCode::UNAUTHORIZED,
                                "parse Authorization header failed",
                            )
                                .into_response();
                        }
                    }
                } else {
                    let msg = format!("parse Authorization header failed: {}", e);
                    warn!(msg);
                    return (
                        StatusCode::UNAUTHORIZED,
                        "parse Authorization header failed",
                    )
                        .into_response();
                }
            }
        };
    let req = match state.verify(&token) {
        Ok(user) => {
            //set the user in the request extensions
            req = Request::from_parts(parts, body);
            req.extensions_mut().insert(user);
            req
        }
        Err(e) => {
            let msg = format!("verify token failed: {:?}", e);
            warn!(msg);
            return (StatusCode::FORBIDDEN, msg).into_response();
        }
    };
    // Call the next middleware in the chain.
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{DecodingKey, EncodingKey, User};

    use super::*;
    use anyhow::Result;
    use axum::{body::Body, middleware::from_fn_with_state, routing::get, Router};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn handler(_req: Request) -> impl IntoResponse {
        (StatusCode::OK, "ok")
    }

    #[derive(Clone)]
    struct AppState(Arc<AppStateInner>);

    struct AppStateInner {
        ek: EncodingKey,
        dk: DecodingKey,
    }

    impl TokenVerifier for AppState {
        type Error = ();
        fn verify(&self, token: &str) -> Result<User, Self::Error> {
            self.0.dk.verify(token).map_err(|_| ())
        }
    }

    #[tokio::test]
    async fn verify_token_should_work() -> Result<()> {
        let ek_str = include_str!("../../fixtures/encoding.pem");
        let dk_str = include_str!("../../fixtures/decoding.pem");
        let state = AppState(Arc::new(AppStateInner {
            ek: EncodingKey::load(ek_str)?,
            dk: DecodingKey::load(dk_str)?,
        }));
        let user = User {
            id: 1,
            name: "Taki".into(),
            email: "Taki@gmail.com".into(),
            created_at: chrono::Utc::now(),
            password_hash: None,
            ws_id: 0,
        };
        let token = state.0.ek.encode(user)?;
        //create a router with the verify_token middleware
        let app = Router::new()
            .route("/", get(handler))
            .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
            .with_state(state);
        //good header token
        //send a request with the token using oneshot
        let req = Request::builder()
            .uri("/")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"ok");

        //good query token
        //send a request with the token using oneshot
        let req = Request::builder()
            .uri(format!("/?access_token={}", token))
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body = res.collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"ok");

        //bad token
        let req = Request::builder()
            .uri("/")
            .header("Authorization", "Bearer bad_token")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
        let body = res.collect().await.unwrap().to_bytes();

        assert_eq!(&body[..], b"verify token failed: ()");

        //no token
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        let body = res.collect().await.unwrap().to_bytes();
        println!("{}", String::from_utf8_lossy(&body));
        assert_eq!(&body[..], b"parse Authorization header failed");
        Ok(())
    }
}
