pub mod ext;
pub mod prelude;

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

pub struct Router<E: Into<Box<dyn StdError + Send + Sync>> + 'static> {
    inner: HashMap<Method, InnerRouter<Box<dyn Handler<E>>>>,
    not_found: Option<Box<dyn Handler<E>>>,
}

impl<E: Into<Box<dyn StdError + Send + Sync>> + 'static> Default for Router<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Into<Box<dyn StdError + Send + Sync>> + 'static> Router<E> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            not_found: None,
        }
    }

    /// Register a handler for GET requests
    pub fn get<H, R>(&mut self, path: &str, handler: H)
    where
        H: Fn(Request<Body>) -> R + Send + Sync + 'static,
        R: Future<Output = std::result::Result<Response<Body>, E>> + Send + Sync + 'static,
        E: Into<Box<dyn StdError + Send + Sync>> + 'static,
    {
        let h = move |req| Box::pin(handler(req));
        let entry = self
            .inner
            .entry(Method::GET)
            .or_insert_with(InnerRouter::new);
        entry.add(path, Box::new(h));
    }

    /// Register a handler for POST requests
    pub fn post<H, R>(&mut self, path: &str, handler: H)
    where
        H: Fn(Request<Body>) -> R + Send + Sync + 'static,
        R: Future<Output = std::result::Result<Response<Body>, E>> + Send + Sync + 'static,
        E: Into<Box<dyn StdError + Send + Sync>> + 'static,
    {
        let h = move |req| Box::pin(handler(req));
        let entry = self
            .inner
            .entry(Method::POST)
            .or_insert_with(InnerRouter::new);
        entry.add(path, Box::new(h));
    }

    /// Register a handler for PUT requests
    pub fn put<H, R>(&mut self, path: &str, handler: H)
    where
        H: Fn(Request<Body>) -> R + Send + Sync + 'static,
        R: Future<Output = std::result::Result<Response<Body>, E>> + Send + Sync + 'static,
        E: Into<Box<dyn StdError + Send + Sync>> + 'static,
    {
        let h = move |req| Box::pin(handler(req));
        let entry = self
            .inner
            .entry(Method::PUT)
            .or_insert_with(InnerRouter::new);
        entry.add(path, Box::new(h));
    }

    /// Register a handler for DELETE requests
    pub fn delete<H, R>(&mut self, path: &str, handler: H)
    where
        H: Fn(Request<Body>) -> R + Send + Sync + 'static,
        R: Future<Output = std::result::Result<Response<Body>, E>> + Send + Sync + 'static,
        E: Into<Box<dyn StdError + Send + Sync>> + 'static,
    {
        let h = move |req| Box::pin(handler(req));
        let entry = self
            .inner
            .entry(Method::DELETE)
            .or_insert_with(InnerRouter::new);
        entry.add(path, Box::new(h));
    }

    /// Register a handler for PATCH requests
    pub fn patch<H, R>(&mut self, path: &str, handler: H)
    where
        H: Fn(Request<Body>) -> R + Send + Sync + 'static,
        R: Future<Output = std::result::Result<Response<Body>, E>> + Send + Sync + 'static,
        E: Into<Box<dyn StdError + Send + Sync>> + 'static,
    {
        let h = move |req| Box::pin(handler(req));
        let entry = self
            .inner
            .entry(Method::PATCH)
            .or_insert_with(InnerRouter::new);
        entry.add(path, Box::new(h));
    }

    /// Register a handler when no routes are matched
    pub fn not_found<H, R>(&mut self, handler: H)
    where
        H: Fn(Request<Body>) -> R + Send + Sync + 'static,
        R: Future<Output = std::result::Result<Response<Body>, E>> + Send + Sync + 'static,
        E: Into<Box<dyn StdError + Send + Sync>> + 'static,
    {
        self.not_found = Some(Box::new(handler));
    }

    pub fn serve(
        &self,
        mut req: Request<Body>,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<Response<Body>, E>> + Send + Sync>>
    where
        E: Into<Box<dyn StdError + Send + Sync>> + 'static,
    {
        match self.inner.get(req.method()) {
            Some(inner_router) => match inner_router.recognize(req.uri().path()) {
                Ok(matcher) => {
                    let handler = matcher.handler();
                    let params = matcher.params().clone();
                    req.extensions_mut().insert(Params(Box::new(params)));
                    handler.call(req)
                }
                Err(_) => match &self.not_found {
                    Some(handler) => handler.call(req),
                    None => Box::pin(async {
                        Ok(Response::builder().status(404).body(Body::empty()).unwrap())
                    }),
                },
            },
            None => {
                Box::pin(async { Ok(Response::builder().status(404).body(Body::empty()).unwrap()) })
            }
        }
    }

    pub fn into_service(self) -> MakeRouterService<RouterService<E>> {
        MakeRouterService {
            inner: RouterService::new(self),
        }
    }
}

pub trait Handler<E: Into<Box<dyn StdError + Send + Sync>>>: Send + Sync + 'static {
    fn call(
        &self,
        req: Request<Body>,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<Response<Body>, E>> + Send + Sync>>;
}

impl<F: Send + Sync + 'static, R, E> Handler<E> for F
where
    F: Fn(Request<Body>) -> R + Send + Sync,
    R: Future<Output = std::result::Result<Response<Body>, E>> + Send + Sync + 'static,
    E: Into<Box<dyn StdError + Send + Sync>>,
{
    fn call(
        &self,
        req: Request<Body>,
    ) -> Pin<Box<dyn Future<Output = std::result::Result<Response<Body>, E>> + Send + Sync>> {
        Box::pin(self(req))
    }
}

impl<E> fmt::Debug for dyn Handler<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "keiro::Handler")
    }
}

#[derive(Clone)]
pub struct RouterService<E: Into<Box<dyn StdError + Send + Sync>> + 'static>(Arc<Router<E>>);

impl<E: Into<Box<dyn StdError + Send + Sync>> + 'static> Service<Request<Body>>
    for RouterService<E>
{
    type Response = Response<Body>;
    type Error = E;
    type Future =
        Pin<Box<dyn Future<Output = std::result::Result<Response<Body>, E>> + Send + Sync>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<std::result::Result<(), E>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        self.0.serve(req)
    }
}

impl<E: Into<Box<dyn StdError + Send + Sync>>> RouterService<E> {
    pub fn new(router: Router<E>) -> Self {
        Self(Arc::new(router))
    }
}

#[derive(Clone)]
pub struct MakeRouterService<Svc> {
    pub inner: Svc,
}

impl<T, Svc> Service<T> for MakeRouterService<Svc>
where
    Svc: Service<Request<Body>> + Clone,
    Svc::Response: 'static,
    Svc::Error: Into<Box<dyn StdError + Send + Sync>>,
    Svc::Future: 'static,
{
    type Response = Svc;
    type Error = Box<dyn StdError + Send + Sync>;
    type Future = futures_util::future::Ready<Result<Self::Response>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _req: T) -> Self::Future {
        futures_util::future::ok(self.inner.clone())
    }
}

pub struct Params(Box<route_recognizer::Params>);

impl Params {
    pub fn find(&self, key: &str) -> Option<&str> {
        self.0.find(key)
    }
}
