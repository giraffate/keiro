use std::net::SocketAddr;

use hyper::{Body, Request, Response, Server};

use keiro::Router;

#[tokio::main]
async fn main() {
    let mut router = Router::new();
    router.get("/", index);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    Server::bind(&addr)
        .serve(router.into_service())
        .await
        .unwrap();
}

async fn index(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    Ok(Response::new(Body::from("Hello, keiro!")))
}
