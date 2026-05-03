use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, ready},
};

use pin_project::pin_project;
use tower::{Layer, Service};
use tracing::{Instrument, Level, debug, error, instrument::Instrumented, span};
use uuid::Uuid;

// LYN: Helpers

type MakeSpanFn<Body> = fn(req: &http::Request<Body>) -> tracing::Span;
type OnRequestFn<Body> = fn(req: &http::Request<Body>, span: &tracing::Span);
type OnResponseFn<Body> =
    fn(resp: &http::Response<Body>, latency: time::Duration, span: &tracing::Span);
type OnErrorFn<Err> = fn(err: &Err, latency: time::Duration, span: &tracing::Span);

fn default_make_span<Body>(_: &http::Request<Body>) -> tracing::Span {
    let id = Uuid::now_v7();
    span!(Level::DEBUG, "request", %id)
}

fn default_on_request<Body>(req: &http::Request<Body>, _: &tracing::Span) {
    debug!(message = "Started processing request", method = %req.method(), uri = %req.uri());
}

fn default_on_response<Body>(
    resp: &http::Response<Body>,
    latency: time::Duration,
    _: &tracing::Span,
) {
    debug!(message = "Finished processing request", status_code = %resp.status(), %latency);
}

fn default_on_error<Err>(err: &Err, latency: time::Duration, _: &tracing::Span)
where
    Err: std::error::Error,
{
    error!(message = "Encountered error while processing request", error = %err, %latency);
}

// LYN: Trace Layer

#[derive(Debug)]
pub struct TraceLayer<Body, Err> {
    make_span: MakeSpanFn<Body>,
    on_request: Option<OnRequestFn<Body>>,
    on_response: Option<OnResponseFn<Body>>,
    on_error: Option<OnErrorFn<Err>>,
}

impl<Body, Err> Clone for TraceLayer<Body, Err> {
    fn clone(&self) -> Self {
        Self {
            make_span: self.make_span,
            on_request: self.on_request,
            on_response: self.on_response,
            on_error: self.on_error,
        }
    }
}

impl<Body, Err: std::fmt::Debug> TraceLayer<Body, Err> {
    pub fn new() -> Self {
        Self {
            make_span: default_make_span,
            on_request: None,
            on_response: None,
            on_error: None,
        }
    }

    pub fn new_with_make_span(make_span: MakeSpanFn<Body>) -> Self {
        Self {
            make_span,
            on_request: None,
            on_response: None,
            on_error: None,
        }
    }

    pub fn make_span(mut self, make_span: MakeSpanFn<Body>) -> Self {
        self.make_span = make_span;
        self
    }

    pub fn on_request(mut self, on_request: OnRequestFn<Body>) -> Self {
        self.on_request = Some(on_request);
        self
    }

    pub fn on_response(mut self, on_response: OnResponseFn<Body>) -> Self {
        self.on_response = Some(on_response);
        self
    }

    pub fn on_error(mut self, on_error: OnErrorFn<Err>) -> Self {
        self.on_error = Some(on_error);
        self
    }
}

impl<Body, Err: std::fmt::Debug> Default for TraceLayer<Body, Err> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Serv, Body, Err> Layer<Serv> for TraceLayer<Body, Err> {
    type Service = TraceService<Serv, Body, Err>;

    fn layer(&self, inner: Serv) -> Self::Service {
        Self::Service {
            inner,

            make_span: self.make_span,
            on_request: self.on_request,
            on_response: self.on_response,
            on_error: self.on_error,
        }
    }
}

// LYN: Trace Service

#[derive(Debug)]
pub struct TraceService<Serv, Body, Err> {
    inner: Serv,

    make_span: MakeSpanFn<Body>,
    on_request: Option<OnRequestFn<Body>>,
    on_response: Option<OnResponseFn<Body>>,
    on_error: Option<OnErrorFn<Err>>,
}

impl<Serv, Body, Err> Clone for TraceService<Serv, Body, Err>
where
    Serv: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),

            make_span: self.make_span,
            on_request: self.on_request,
            on_response: self.on_response,
            on_error: self.on_error,
        }
    }
}

impl<Serv, Body, Err> Service<http::Request<Body>> for TraceService<Serv, Body, Err>
where
    Serv: Service<http::Request<Body>, Response = http::Response<Body>, Error = Err>,
    Err: std::error::Error,
{
    type Response = Serv::Response;
    type Error = Serv::Error;
    type Future = TraceServiceFuture<Serv::Future, Body, Err>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<Body>) -> Self::Future {
        let start = time::OffsetDateTime::now_utc();

        let span = (self.make_span)(&req);

        let resp_fut = {
            let on_request = self.on_request.unwrap_or(default_on_request);
            span.in_scope(|| on_request(&req, &span));
            self.inner.call(req).instrument(span.clone())
        };

        TraceServiceFuture {
            resp_fut,
            start,
            span,

            on_response: self.on_response,
            on_error: self.on_error,
        }
    }
}

// LYN: Trace Service Future

#[derive(Debug)]
#[pin_project]
pub struct TraceServiceFuture<Fut, Body, Err> {
    #[pin]
    resp_fut: Instrumented<Fut>,
    start: time::OffsetDateTime,
    span: tracing::Span,

    on_response: Option<OnResponseFn<Body>>,
    on_error: Option<OnErrorFn<Err>>,
}

impl<Fut, Body, Err> Future for TraceServiceFuture<Fut, Body, Err>
where
    Fut: Future<Output = Result<http::Response<Body>, Err>>,
    Err: std::error::Error,
{
    type Output = Result<http::Response<Body>, Err>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let result = ready!(this.resp_fut.poll(cx));
        let latency = {
            let elapsed = time::OffsetDateTime::now_utc() - *this.start;
            if elapsed.is_negative() {
                time::Duration::ZERO
            } else {
                elapsed
            }
        };

        match result {
            Ok(resp) => {
                let on_response = this.on_response.unwrap_or(default_on_response);
                this.span
                    .in_scope(|| on_response(&resp, latency, this.span));
                Poll::Ready(Ok(resp))
            }
            Err(err) => {
                let on_error = this.on_error.unwrap_or(default_on_error);
                this.span.in_scope(|| on_error(&err, latency, this.span));
                Poll::Ready(Err(err))
            }
        }
    }
}
