use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll, ready},
};

use pin_project::pin_project;
use tower::{Layer, Service};

// LYN: Helpers

type MakeSpanFn<Body> = fn(req: &http::Request<Body>) -> tracing::Span;
type OnRequestFn<Body> = fn(req: &http::Request<Body>, span: &tracing::Span);
type OnResponseFn<Body> =
    fn(resp: &http::Response<Body>, elapsed: web_time::Duration, span: &tracing::Span);
type OnErrorFn<Err> = fn(err: &Err, elapsed: web_time::Duration, span: &tracing::Span);

// LYN: Trace Layer

#[derive(Debug)]
pub struct TraceLayer<Body, Err> {
    make_span: MakeSpanFn<Body>,
    on_request: Option<OnRequestFn<Body>>,
    on_response: Option<OnResponseFn<Body>>,
    on_error: Option<OnErrorFn<Err>>,

    _marker_body: PhantomData<fn() -> Body>,
    _marker_err: PhantomData<fn() -> Err>,
}

impl<Body, Err> Clone for TraceLayer<Body, Err> {
    fn clone(&self) -> Self {
        Self {
            make_span: self.make_span,
            on_request: self.on_request,
            on_response: self.on_response,
            on_error: self.on_error,

            _marker_body: PhantomData,
            _marker_err: PhantomData,
        }
    }
}

impl<Body, Err: std::fmt::Debug> TraceLayer<Body, Err> {
    pub fn new(make_span: MakeSpanFn<Body>) -> Self {
        Self {
            make_span,
            on_request: None,
            on_response: None,
            on_error: None,

            _marker_body: PhantomData,
            _marker_err: PhantomData,
        }
    }

    #[allow(unused)]
    pub fn make_span(mut self, make_span: MakeSpanFn<Body>) -> Self {
        self.make_span = make_span;
        self
    }

    #[allow(unused)]
    pub fn on_request(mut self, on_request: OnRequestFn<Body>) -> Self {
        self.on_request = Some(on_request);
        self
    }

    #[allow(unused)]
    pub fn on_response(mut self, on_response: OnResponseFn<Body>) -> Self {
        self.on_response = Some(on_response);
        self
    }

    #[allow(unused)]
    pub fn on_error(mut self, on_error: OnErrorFn<Err>) -> Self {
        self.on_error = Some(on_error);
        self
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

            _marker_body: PhantomData,
            _marker_err: PhantomData,
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

    _marker_body: PhantomData<fn() -> Body>,
    _marker_err: PhantomData<fn() -> Err>,
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

            _marker_body: PhantomData,
            _marker_err: PhantomData,
        }
    }
}

impl<Serv, Body, Err: std::fmt::Debug> Service<http::Request<Body>>
    for TraceService<Serv, Body, Err>
where
    Serv: Service<http::Request<Body>, Response = http::Response<Body>, Error = Err>,
{
    type Response = Serv::Response;
    type Error = Serv::Error;
    type Future = TraceServiceFuture<Serv::Future, Body, Err>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<Body>) -> Self::Future {
        let start = web_time::Instant::now();

        let span = (self.make_span)(&req);

        let resp_fut = {
            let _guard = span.enter();
            if let Some(on_request) = self.on_request {
                on_request(&req, &span);
            }
            self.inner.call(req)
        };

        TraceServiceFuture {
            resp_fut,
            start,
            span,

            on_response: self.on_response,
            on_error: self.on_error,

            _marker_body: PhantomData,
            _marker_err: PhantomData,
        }
    }
}

// LYN: Trace Service Future

#[derive(Debug)]
#[pin_project]
pub struct TraceServiceFuture<Fut, Body, Err> {
    #[pin]
    resp_fut: Fut,
    start: web_time::Instant,
    span: tracing::Span,

    on_response: Option<OnResponseFn<Body>>,
    on_error: Option<OnErrorFn<Err>>,

    _marker_body: PhantomData<fn() -> Body>,
    _marker_err: PhantomData<fn() -> Err>,
}

impl<Fut, Body, Err> Future for TraceServiceFuture<Fut, Body, Err>
where
    Fut: Future<Output = Result<http::Response<Body>, Err>>,
{
    type Output = Result<http::Response<Body>, Err>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _guard = this.span.enter();
        let result = ready!(this.resp_fut.poll(cx));
        let elapsed = this.start.elapsed();

        match result {
            Ok(resp) => {
                if let Some(on_response) = this.on_response.take() {
                    on_response(&resp, elapsed, this.span);
                }
                Poll::Ready(Ok(resp))
            }
            Err(err) => {
                if let Some(on_error) = this.on_error.take() {
                    on_error(&err, elapsed, this.span);
                }
                Poll::Ready(Err(err))
            }
        }
    }
}
