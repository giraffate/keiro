use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use hyper::service::Service;
use hyper::{Body, Method, Request, Response};
use route_recognizer::Router as InnerRouter;

pub struct Router {
    inner: HashMap<Method, InnerRouter<Box<dyn Handler>>>,
}

impl Router {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn get(&mut self, path: &str, handler: impl Handler) {
        let entry = self.inner.entry(Method::GET).or_insert(InnerRouter::new());
        entry.add(path, Box::new(handler));
    }

    pub fn serve(
        &self,
        req: Request<Body>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Body>, hyper::Error>> + Send + Sync>> {
        match self.inner.get(req.method()) {
            Some(inner_router) => match inner_router.recognize(req.uri().path()) {
                Ok(matcher) => {
                    let handler = matcher.handler();
                    let _params = matcher.params();
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
