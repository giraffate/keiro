use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::{Body, Request, Response, Server};
use keiro::Router;

#[derive(Clone)]
struct State {
  name: String,
}

#[tokio::main]
async fn main() {
    let state = State {
        name: "giraffate".to_string(),
    };
    let mut router = Router::with_state(state);
    router.get("/", index);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    Server::bind(&addr)
        .serve(router.into_service())
        .await
        .unwrap();
}

async fn index(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let state = req.extensions().get::<State>().unwrap();
    Ok(Response::new(Body::from(format!("Hello {}", state.name))))
}
