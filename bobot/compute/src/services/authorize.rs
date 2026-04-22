use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use axum::{body::to_bytes, response::IntoResponse};
use pengu::bot::BotClient;
use tower::{Layer, Service};
use tracing::{debug, warn};

// LYN: Helpers

type MakeSpanFn = fn(req: &http::Request<axum::body::Body>) -> tracing::Span;

// LYN: Authorize Layer

#[derive(Debug, Clone)]
pub struct AuthoriseLayer {
    qqbot: BotClient,
    make_span: MakeSpanFn,
}

impl AuthoriseLayer {
    pub fn new(qqbot: BotClient, make_span: MakeSpanFn) -> Self {
        Self { qqbot, make_span }
    }
}

impl<Serv> Layer<Serv> for AuthoriseLayer {
    type Service = AuthoriseService<Serv>;

    fn layer(&self, inner: Serv) -> Self::Service {
        AuthoriseService {
            inner,
            qqbot: self.qqbot.clone(),
            make_span: self.make_span,
        }
    }
}

// LYN: Authorize Service

#[derive(Debug, Clone)]
pub struct AuthoriseService<Serv> {
    inner: Serv,
    qqbot: BotClient,
    make_span: MakeSpanFn,
}

impl<Serv> Service<http::Request<axum::body::Body>> for AuthoriseService<Serv>
where
    Serv: Service<http::Request<axum::body::Body>, Response = axum::response::Response>
        + Clone
        + Send
        + 'static,
    Serv::Future: Send,
    Serv::Error: std::fmt::Display,
{
    type Response = Serv::Response;
    type Error = Serv::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<axum::body::Body>) -> Self::Future {
        let mut inner = self.inner.clone();
        let qqbot = self.qqbot.clone();
        let make_span = self.make_span;

        Box::pin(async move {
            let span = make_span(&req);
            let (parts, body) = req.into_parts();

            let Some(sig) = parts
                .headers
                .get(BotClient::HEADER_SIGNATURE_STRING)
                .and_then(|v| v.to_str().ok())
            else {
                span.in_scope(|| debug!(message = "reject because no signature string header"));
                return Ok(http::StatusCode::UNAUTHORIZED.into_response());
            };

            let Some(ts) = parts
                .headers
                .get(BotClient::HEADER_SIGNATURE_TIMESTAMP)
                .and_then(|v| v.to_str().ok())
            else {
                span.in_scope(|| debug!(message = "reject because no signature timestamp header"));
                return Ok(http::StatusCode::UNAUTHORIZED.into_response());
            };

            let body = match to_bytes(body, usize::MAX).await {
                Ok(body) => body,
                Err(err) => {
                    span.in_scope(|| warn!(message = "failed to read body", error = %err));
                    return Ok(http::StatusCode::BAD_REQUEST.into_response());
                }
            };

            let mut message = Vec::new();
            message.extend_from_slice(ts.as_bytes());
            message.extend_from_slice(body.iter().as_slice());
            if !qqbot.validate_signature(message.iter().as_slice(), sig) {
                span.in_scope(|| debug!(message = "reject because signature validation failed"));
                return Ok(http::StatusCode::UNAUTHORIZED.into_response());
            }

            let req = http::Request::from_parts(parts, axum::body::Body::from(body));
            inner.call(req).await
        })
    }
}
