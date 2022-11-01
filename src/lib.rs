//! Keiro is a lightweight router for Rust HTTP services. It is based on [hyper](https://github.com/hyperium/hyper).
//!
//! ## Usage
//! ```rust,no_run
//! use std::convert::Infallible;
//! use std::net::SocketAddr;
//!
//! use hyper::{Body, Request, Response, Server};
//! use keiro::prelude::*;
//! use keiro::Router;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut router = Router::new();
//!     router.get("/", index);
//!     router.get("/hello/:user1/from/:user2", hello);
//!     router.get("/hi/*path", hi);
//!     let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
//!
//!     Server::bind(&addr)
//!         .serve(router.into_service())
//!         .await
//!         .unwrap();
//! }
//!
//! async fn index(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
//!     Ok(Response::new(Body::from("Hello keiro!")))
//! }
//!
//! async fn hello(req: Request<Body>) -> Result<Response<Body>, Infallible> {
//!     let params = req.params().unwrap();
//!     Ok(Response::new(Body::from(format!(
//!         "Hello {} from {}!",
//!         params.find("user1").unwrap(),
//!         params.find("user2").unwrap(),
//!     ))))
//! }
//!
//! async fn hi(req: Request<Body>) -> Result<Response<Body>, Infallible> {
//!     let params = req.params().unwrap();
//!     Ok(Response::new(Body::from(format!(
//!         "Hello {}!",
//!         params.find("path").unwrap(),
//!     ))))
//! }
//! ```
//!
//! ### Routing
//!
//! Keiro uses [`route-recognier`](https://github.com/http-rs/route-recognizer) and supports
//! four kinds of route segments:
//! - segments: these are of the format `/a/b`.
//! - params: these are of the format `/a/:b`.
//! - named wildcards: these are of the format `/a/*b`.
//! - unnamed wildcards: these are of the format `/a/*`.
//!
//! ### Middleware
//!
//! [`tower-http`](https://github.com/tower-rs/tower-http) supports useful middelwares and Keiro can use them.
//! ```rust,no_run
//! use std::convert::Infallible;
//! use std::net::SocketAddr;
//!
//! use hyper::{Body, Request, Response, Server};
//! use keiro::prelude::*;
//! use keiro::{Params, Router};
//! use tower::{make::Shared, ServiceBuilder};
//! use tower_http::trace::TraceLayer;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut router = Router::new();
//!     router.get("/", index);
//!     let svc = keiro::RouterService::new(router);
//!
//!     tracing_subscriber::fmt::init();
//!     let service = ServiceBuilder::new()
//!         .layer(TraceLayer::new_for_http())
//!         .service(svc);
//!
//!     let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
//!
//!     Server::bind(&addr)
//!         .serve(Shared::new(service))
//!         .await
//!         .unwrap();
//! }
//!
//! async fn index(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
//!     Ok(Response::new(Body::from("Hello keiro!")))
//! }
//! ```
//!
//! ### Not found handler
//!
//! To handle requests which couldn't be matched by Keiro, `not_found` handler can be used.
//! ```rust,no_run
//! use std::convert::Infallible;
//! use std::net::SocketAddr;
//!
//! use hyper::{Body, Request, Response, Server};
//! use keiro::Router;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut router = Router::new();
//!     router.get("/", index);
//!     router.not_found(not_found);
//!     let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
//!
//!     Server::bind(&addr)
//!         .serve(router.into_service())
//!         .await
//!         .unwrap();
//! }
//!
//! async fn index(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
//!     Ok(Response::new(Body::from("Hello keiro!")))
//! }
//!
//! async fn not_found(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
//!     let response = Response::builder()
//!         .status(404)
//!         .body(Body::from("Not Found!"))
//!         .unwrap();
//!     Ok(response)
//! }
//! ```
//!
//! ### Share states
//!
//! Handler can use share states.
//! ```rust,no_run
//! use std::convert::Infallible;
//! use std::net::SocketAddr;
//!
//! use hyper::{Body, Request, Response, Server};
//! use keiro::prelude::*;
//! use keiro::Router;
//!
//! #[derive(Clone)]
//! struct State {
//!     name: String,
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let state = State {
//!         name: "giraffate".to_string(),
//!     };
//!     let mut router = Router::with_state(state);
//!     router.get("/", index);
//!     let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
//!
//!     Server::bind(&addr)
//!         .serve(router.into_service())
//!         .await
//!         .unwrap();
//! }
//!
//! async fn index(req: Request<Body>) -> Result<Response<Body>, Infallible> {
//!     let state = req.state::<State>().unwrap();
//!     Ok(Response::new(Body::from(format!("Hello {}", state.name))))
//! }
//! ```

pub mod ext;
pub mod prelude;

use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use hyper::service::Service;
use hyper::{Body, Method, Request, Response};
use route_recognizer::Router as InnerRouter;

pub struct Router<E, State> {
    inner: HashMap<Method, InnerRouter<Box<dyn Handler<E>>>>,
    not_found: Option<Box<dyn Handler<E>>>,
    state: State,
}

impl<E> Default for Router<E, ()>
where
    E: Into<Box<dyn Error + Send + Sync>> + 'static,
{
    fn default() -> Self {
        let b = 0;
        Self::new()
    }
}

impl<E> Router<E, ()>
where
    E: Into<Box<dyn Error + Send + Sync>> + 'static,
{
    pub fn new() -> Self {
        Router::with_state(())
    }
}

impl<E, State> Router<E, State>
where
    E: Into<Box<dyn Error + Send + Sync>> + 'static,
    State: Clone + Send + Sync + 'static,
{
    pub fn with_state(state: State) -> Self {
        Self {
            inner: HashMap::new(),
            not_found: None,
            state,
        }
    }

    /// Register a handler for GET requests
    pub fn get<H, R>(&mut self, path: &str, handler: H)
    where
        H: Fn(Request<Body>) -> R + Send + Sync + 'static,
        R: Future<Output = Result<Response<Body>, E>> + Send + Sync + 'static,
        E: Into<Box<dyn Error + Send + Sync>> + 'static,
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
        R: Future<Output = Result<Response<Body>, E>> + Send + Sync + 'static,
        E: Into<Box<dyn Error + Send + Sync>> + 'static,
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
        R: Future<Output = Result<Response<Body>, E>> + Send + Sync + 'static,
        E: Into<Box<dyn Error + Send + Sync>> + 'static,
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
        R: Future<Output = Result<Response<Body>, E>> + Send + Sync + 'static,
        E: Into<Box<dyn Error + Send + Sync>> + 'static,
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
        R: Future<Output = Result<Response<Body>, E>> + Send + Sync + 'static,
        E: Into<Box<dyn Error + Send + Sync>> + 'static,
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
        R: Future<Output = Result<Response<Body>, E>> + Send + Sync + 'static,
        E: Into<Box<dyn Error + Send + Sync>> + 'static,
    {
        self.not_found = Some(Box::new(handler));
    }

    pub fn serve(
        &self,
        mut req: Request<Body>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Body>, E>> + Send + Sync>>
    where
        E: Into<Box<dyn Error + Send + Sync>> + 'static,
    {
        match self.inner.get(req.method()) {
            Some(inner_router) => match inner_router.recognize(req.uri().path()) {
                Ok(matcher) => {
                    let handler = matcher.handler();
                    let params = matcher.params().clone();
                    req.extensions_mut().insert(Params(Box::new(params)));
                    req.extensions_mut().insert(self.state.clone());
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

    pub fn into_service(self) -> MakeRouterService<RouterService<E, State>> {
        MakeRouterService {
            inner: RouterService::new(self),
        }
    }
}

pub trait Handler<E: Into<Box<dyn Error + Send + Sync>>>: Send + Sync + 'static {
    fn call(
        &self,
        req: Request<Body>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Body>, E>> + Send + Sync>>;
}

impl<F: Send + Sync + 'static, R, E> Handler<E> for F
where
    F: Fn(Request<Body>) -> R + Send + Sync,
    R: Future<Output = Result<Response<Body>, E>> + Send + Sync + 'static,
    E: Into<Box<dyn Error + Send + Sync>>,
{
    fn call(
        &self,
        req: Request<Body>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Body>, E>> + Send + Sync>> {
        Box::pin(self(req))
    }
}

impl<E> fmt::Debug for dyn Handler<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "keiro::Handler")
    }
}

#[derive(Clone)]
pub struct RouterService<E, State>(Arc<Router<E, State>>);

impl<E, State> Service<Request<Body>> for RouterService<E, State>
where
    E: Into<Box<dyn Error + Send + Sync>> + 'static,
    State: Clone + Send + Sync + 'static,
{
    type Response = Response<Body>;
    type Error = Box<dyn Error + Send + Sync>;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + Sync>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let router = self.0.clone();
        let fut = router.serve(req);
        let fut = async { fut.await.map_err(Into::into) };
        Box::pin(fut)
    }
}

impl<E, State> RouterService<E, State>
where
    E: Into<Box<dyn Error + Send + Sync>> + 'static,
    State: Clone + Send + Sync + 'static,
{
    pub fn new(router: Router<E, State>) -> Self {
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
    Svc::Error: Into<Box<dyn Error + Send + Sync>>,
    Svc::Future: 'static,
{
    type Response = Svc;
    type Error = Box<dyn Error + Send + Sync>;
    type Future = futures_util::future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
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
