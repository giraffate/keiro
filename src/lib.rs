use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use hyper::service::Service;
use hyper::{Body, Method, Request, Response};
use route_recognizer::Router as InnerRouter;

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
    ) -> Pin<Box<dyn Future<Output = Result<Response<Body>, hyper::Error>> + Send + Sync>> {
        match self.inner.get(req.method()) {
            Some(inner_router) => match inner_router.recognize(req.uri().path()) {
                Ok(matcher) => {
                    let handler = matcher.handler();
                    let params = matcher.params().clone();
                    req.extensions_mut().insert(Params(Box::new(params)));
                    handler.call(req)
                }
                Err(_) => Box::pin(async {
                    Ok(Response::builder().status(404).body(Body::empty()).unwrap())
                }),
            },
            None => {
                Box::pin(async { Ok(Response::builder().status(404).body(Body::empty()).unwrap()) })
            }
        }
    }

    pub fn into_service(self) -> MakeRouterService {
        MakeRouterService(RouterService::new(self))
    }
}

pub trait Handler: Send + Sync + 'static {
    fn call(
        &self,
        req: Request<Body>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Body>, hyper::Error>> + Send + Sync>>;
}

impl<F: Send + Sync + 'static, R> Handler for F
where
    F: Fn(Request<Body>) -> R + Send + Sync,
    R: Future<Output = Result<Response<Body>, hyper::Error>> + Send + Sync + 'static,
{
    fn call(
        &self,
        req: Request<Body>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Body>, hyper::Error>> + Send + Sync>> {
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
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Response<Body>, hyper::Error>> + Send + Sync>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        self.0.serve(req)
    }
}

impl RouterService {
    fn new(router: Router) -> Self {
        Self(Arc::new(router))
    }
}

pub struct MakeRouterService(RouterService);

impl<T> Service<T> for MakeRouterService {
    type Response = RouterService;
    type Error = hyper::Error;
    type Future = futures_util::future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: T) -> Self::Future {
        futures_util::future::ok(self.0.clone())
    }
}

pub struct Params(Box<route_recognizer::Params>);

impl Params {
    pub fn find(&self, key: &str) -> Option<&str> {
        self.0.find(key)
    }
}
