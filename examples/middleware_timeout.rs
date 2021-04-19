use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use hyper::service::Service;
use hyper::{Body, Request, Response, Server};
use keiro::Router;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let mut router = Router::new();
    router.get("/", index);

    let svc = keiro::RouterService::new(router);
    let svc = Timeout::new(svc, Duration::from_secs(3));

    Server::bind(&addr)
        .serve(keiro::MakeRouterService { inner: svc })
        .await
        .unwrap();
}

async fn index(_req: Request<Body>) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
    sleep(tokio::time::Duration::from_secs(5)).await;
    Ok(Response::new(Body::from("Hello keiro!")))
}

#[derive(Clone)]
pub struct Timeout<Svc> {
    inner: Svc,
    timeout: Duration,
}

impl<Svc> Timeout<Svc> {
    pub fn new(inner: Svc, timeout: Duration) -> Timeout<Svc> {
        Timeout { inner, timeout }
    }
}

impl<Svc> Service<Request<Body>> for Timeout<Svc>
where
    Svc: Service<Request<Body>> + Clone,
    Svc::Response: Into<Response<Body>> + Send + Sync + 'static,
    Svc::Error: Into<Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static,
    Svc::Future: Send + Sync + 'static,
{
    type Response = Response<Body>;
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Future = Pin<Box<dyn Future<Output = keiro::Result<Response<Body>>> + Send + Sync>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let timeout = tokio::time::sleep(self.timeout);
        let fut = self.inner.call(req);

        Box::pin(async move {
            tokio::select! {
                res = fut => {
                    res.map(|v| v.into()).map_err(|err| err.into())
                },
                _ = timeout => {
                    Ok(Response::builder().status(500).body(Body::from("expired")).unwrap())
                },
            }
        })
    }
}
