use std::error::Error;
use std::net::SocketAddr;

use hyper::{Body, Request, Response, Server};
use keiro::{Params, Router};

#[tokio::main]
async fn main() {
    let mut router = Router::new();
    router.get("/", index);
    router.get("/hello/:user1/from/:user2", hello);
    router.get("/hi/*path", hi);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    Server::bind(&addr)
        .serve(router.into_service())
        .await
        .unwrap();
}

async fn index(_req: Request<Body>) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
    Ok(Response::new(Body::from("Hello keiro!")))
}

async fn hello(req: Request<Body>) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
    let params = req.extensions().get::<Params>().unwrap();
    Ok(Response::new(Body::from(format!(
        "Hello {} from {}!",
        params.find("user1").unwrap(),
        params.find("user2").unwrap(),
    ))))
}

async fn hi(req: Request<Body>) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
    let params = req.extensions().get::<Params>().unwrap();
    Ok(Response::new(Body::from(format!(
        "Hello {}!",
        params.find("path").unwrap(),
    ))))
}
