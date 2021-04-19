use std::error::Error;
use std::net::SocketAddr;

use hyper::{Body, Request, Response, Server};
use keiro::Router;

#[tokio::main]
async fn main() {
    let mut router = Router::new();
    router.get("/", index);
    router.not_found(not_found);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    Server::bind(&addr)
        .serve(router.into_service())
        .await
        .unwrap();
}

async fn index(_req: Request<Body>) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
    Ok(Response::new(Body::from("Hello keiro!")))
}

async fn not_found(_req: Request<Body>) -> Result<Response<Body>, Box<dyn Error + Send + Sync>> {
    let response = Response::builder()
        .status(404)
        .body(Body::from("Not Found!"))
        .unwrap();
    Ok(response)
}
