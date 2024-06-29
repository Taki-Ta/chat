use crate::AppState;
use axum::{
    extract::{FromRequestParts, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use tracing::warn;

pub async fn verify_token(State(state): State<AppState>, mut req: Request, next: Next) -> Response {
    //get token from the request headers and validate it
    let (mut parts, body) = req.into_parts();
    let req =
        match TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await {
            Ok(TypedHeader(Authorization(bearer))) => {
                let token = bearer.token();
                //verify the token
                match state.dk.verify(token) {
                    Ok(user) => {
                        //set the user in the request extensions
                        req = Request::from_parts(parts, body);
                        req.extensions_mut().insert(user);
                        req
                    }
                    Err(e) => {
                        let msg = format!("verify token failed: {}", e);
                        warn!(msg);
                        return (StatusCode::FORBIDDEN, msg).into_response();
                    }
                }
            }
            Err(e) => {
                let msg = format!("parse Authorization header failed: {}", e);
                warn!(msg);
                return (StatusCode::UNAUTHORIZED, msg).into_response();
            }
        };
    // Call the next middleware in the chain.
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use crate::User;

    use super::*;
    use anyhow::Result;
    use axum::{body::Body, middleware::from_fn_with_state, routing::get, Router};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    async fn handler(_req: Request) -> impl IntoResponse {
        (StatusCode::OK, "ok")
    }

    #[tokio::test]
    async fn verify_token_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let user = User::new(1, "Taki", "Taki@gmail.com");
        let token = state.ek.encode(user)?;
        //create a router with the verify_token middleware
        let app = Router::new()
            .route("/", get(handler))
            .layer(from_fn_with_state(state.clone(), verify_token))
            .with_state(state);
        //good token
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

        //bad token
        let req = Request::builder()
            .uri("/")
            .header("Authorization", "Bearer bad_token")
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
        let body = res.collect().await.unwrap().to_bytes();

        assert_eq!(
            &body[..],
            b"verify token failed: jwt sign error: JWT compact encoding error"
        );

        //no token
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = app.clone().oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        let body = res.collect().await.unwrap().to_bytes();
        assert_eq!(
            &body[..],
            b"parse Authorization header failed: Header of type `authorization` was missing"
        );
        Ok(())
    }
}
