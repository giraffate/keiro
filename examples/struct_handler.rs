use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;

use hyper::{Body, Request, Response, Server};
use keiro::{Handler, Router};

#[tokio::main]
async fn main() {
    let mut router = Router::new();
    let index_handler = IndexHandler {
        message: "Hello from a struct handler!".to_string(),
    };
    router.get("/", index_handler);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    Server::bind(&addr)
        .serve(router.into_service())
        .await
        .unwrap();
}

struct IndexHandler {
    message: String,
}

impl Handler for IndexHandler {
    fn call(
        &self,
        _req: Request<Body>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Body>, hyper::Error>> + Send + Sync>> {
        let message = self.message.clone();
        Box::pin(async { Ok(Response::new(Body::from(message))) })
    }
}
