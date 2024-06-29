use super::REQUEST_ID_HEADER;
use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use tracing::warn;

pub async fn set_request_id(mut req: Request, next: Next) -> Response {
    let id = match req.headers().get(REQUEST_ID_HEADER) {
        // If the request already has a request id, just clone it and return it.
        Some(v) => Some(v.clone()),
        // If the request does not have a request id, generate a new one and set it in the request headers.
        None => {
            let request_id = uuid::Uuid::now_v7().to_string();
            match HeaderValue::from_str(&request_id) {
                // If the request id is successfully generated, set it in the request headers and return it.
                Ok(v) => {
                    req.headers_mut().insert(REQUEST_ID_HEADER, v.clone());
                    Some(v)
                }
                // If the request id generation fails, log the error and return None.
                Err(e) => {
                    warn!("parse generated request id failed: {}", e);
                    None
                }
            }
        }
    };
    // Call the next middleware in the chain.
    let mut res = next.run(req).await;

    let Some(id) = id else {
        return res;
    };
    // Set the request id in the response headers.
    res.headers_mut().insert(REQUEST_ID_HEADER, id);
    res
}
