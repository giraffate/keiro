use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use hyper::service::Service;
use hyper::{Body, Method, Request, Response};
use route_recognizer::Router as InnerRouter;

pub type Result<T> = std::result::Result<T, Box<dyn StdError + Send + Sync>>;

#[derive(Debug)]
pub struct Error {
    inner: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl StdError for Error {}

#[derive(Debug)]
pub struct Router {
    inner: HashMap<Method, InnerRouter<Box<dyn Handler>>>,
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

impl Router {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Register a handler for GET requests
    pub fn get(&mut self, path: &str, handler: impl Handler) {
        let entry = self
            .inner
            .entry(Method::GET)
            .or_insert_with(InnerRouter::new);
        entry.add(path, Box::new(handler));
    }

    /// Register a handler for POST requests
    pub fn post(&mut self, path: &str, handler: impl Handler) {
        let entry = self
            .inner
            .entry(Method::POST)
            .or_insert_with(InnerRouter::new);
        entry.add(path, Box::new(handler));
    }

    /// Register a handler for PUT requests
    pub fn put(&mut self, path: &str, handler: impl Handler) {
        let entry = self
            .inner
            .entry(Method::PUT)
            .or_insert_with(InnerRouter::new);
        entry.add(path, Box::new(handler));
    }

    /// Register a handler for DELETE requests
    pub fn delete(&mut self, path: &str, handler: impl Handler) {
        let entry = self
            .inner
            .entry(Method::DELETE)
            .or_insert_with(InnerRouter::new);
        entry.add(path, Box::new(handler));
    }

    /// Register a handler for PATCH requests
    pub fn patch(&mut self, path: &str, handler: impl Handler) {
        let entry = self
            .inner
            .entry(Method::PATCH)
            .or_insert_with(InnerRouter::new);
        entry.add(path, Box::new(handler));
    }

    pub fn serve(
        &self,
        mut req: Request<Body>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Body>>> + Send + Sync>> {
        match self.inner.get(req.method()) {
            Some(inner_router) => match inner_router.recognize(req.uri().path()) {
                Ok(matcher) => {
                    let handler = matcher.handler();
                    let params = matcher.params().clone();
                    req.extensions_mut().insert(Params(Box::new(params)));
                    handler.call(req)
                }
                Err(e) => Box::pin(async {
                    Err(Box::new(Error { inner: e }) as Box<dyn StdError + Send + Sync>)
                }),
            },
            None => {
                Box::pin(async { Ok(Response::builder().status(404).body(Body::empty()).unwrap()) })
            }
        }
    }

    pub fn into_service(self) -> MakeRouterService<RouterService> {
        MakeRouterService {
            inner: RouterService::new(self),
        }
    }
}

pub trait Handler: Send + Sync + 'static {
    fn call(
        &self,
        req: Request<Body>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Body>>> + Send + Sync>>;
}

impl<F: Send + Sync + 'static, R> Handler for F
where
    F: Fn(Request<Body>) -> R + Send + Sync,
    R: Future<Output = Result<Response<Body>>> + Send + Sync + 'static,
{
    fn call(
        &self,
        req: Request<Body>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Body>>> + Send + Sync>> {
        Box::pin(self(req))
    }
}

impl fmt::Debug for dyn Handler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "keiro::Handler")
    }
}

#[derive(Clone)]
pub struct RouterService(Arc<Router>);

impl Service<Request<Body>> for RouterService {
    type Response = Response<Body>;
    type Error = Box<dyn StdError + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = Result<Response<Body>>> + Send + Sync>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        self.0.serve(req)
    }
}

impl RouterService {
    pub fn new(router: Router) -> Self {
        Self(Arc::new(router))
    }
}

pub struct MakeRouterService<Svc> {
    pub inner: Svc,
}

impl<T, Svc> Service<T> for MakeRouterService<Svc>
where
    Svc: Service<Request<Body>> + Clone,
    Svc::Response: 'static,
    Svc::Error: Into<Box<dyn StdError + Send + Sync>> + 'static,
    Svc::Future: 'static,
{
    type Response = Svc;
    type Error = Box<dyn StdError + Send + Sync>;
    type Future = futures_util::future::Ready<Result<Self::Response>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: T) -> Self::Future {
        // TODO: address err
        futures_util::future::ok(self.inner.clone())
    }
}

pub struct Params(Box<route_recognizer::Params>);

impl Params {
    pub fn find(&self, key: &str) -> Option<&str> {
        self.0.find(key)
    }
}
