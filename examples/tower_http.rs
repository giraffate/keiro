use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::{Body, Request, Response, Server};
use keiro::prelude::*;
use keiro::{Params, Router};
use tower::{make::Shared, ServiceBuilder};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    let mut router = Router::new();
    router.get("/", index);
    router.get("/hello/:user1/from/:user2", hello);
    router.get("/hi/*path", hi);
    let svc = keiro::RouterService::new(router);

    tracing_subscriber::fmt::init();
    let service = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .service(svc);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    Server::bind(&addr)
        .serve(Shared::new(service))
        .await
        .unwrap();
}

async fn index(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Hello keiro!")))
}

async fn hello(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let params = req.extensions().get::<Params>().unwrap();
    Ok(Response::new(Body::from(format!(
        "Hello {} from {}!",
        params.find("user1").unwrap(),
        params.find("user2").unwrap(),
    ))))
}

async fn hi(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let params = req.params().unwrap();
    Ok(Response::new(Body::from(format!(
        "Hello {}!",
        params.find("path").unwrap(),
    ))))
}
