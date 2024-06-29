use super::REQUEST_ID_HEADER;
use axum::{extract::Request, response::Response};
use futures_util::future::BoxFuture;
use std::task::{Context, Poll};
use tokio::time::Instant;
use tower::{Layer, Service};
use tracing::warn;

#[derive(Clone)]
pub struct ServerTimeLayer;

impl<S> Layer<S> for ServerTimeLayer {
    type Service = ServerTimeMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ServerTimeMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct ServerTimeMiddleware<S> {
    pub inner: S,
}

impl<S> Service<Request> for ServerTimeMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        //start the timer
        let start = Instant::now();
        let future = self.inner.call(request);
        Box::pin(async move {
            let mut response: Response = future.await?;
            //calculate the elapsed time
            let elapsed = format!("{}μs", start.elapsed().as_micros());
            match elapsed.parse() {
                Ok(v) => {
                    //set the elapsed time in the response headers
                    response.headers_mut().insert("x-server-time", v);
                }
                Err(e) => {
                    warn!(
                        "Parse elapsed time failed: {} for request {:?}",
                        e,
                        response.headers().get(REQUEST_ID_HEADER)
                    );
                }
            }
            Ok(response)
        })
    }
}
