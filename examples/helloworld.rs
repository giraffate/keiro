use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;

use hyper::{Body, Request, Response, Server};

use keiro::{Handler, Params, Router};

#[tokio::main]
async fn main() {
    let mut router = Router::new();
    let about_handler = AboutHandler {
        message: "Hello from a struct handler!".to_string(),
    };
    router.get("/", index);
    router.get("/hello/:user", hello);
    router.get("/about", about_handler);
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    Server::bind(&addr)
        .serve(router.into_service())
        .await
        .unwrap();
}

async fn index(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    Ok(Response::new(Body::from("Hello, keiro!")))
}

async fn hello(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let params = req.extensions().get::<Params>().unwrap();
    Ok(Response::new(Body::from(format!(
        "Hello, {}!",
        params.find("user").unwrap()
    ))))
}

struct AboutHandler {
    message: String,
}

impl Handler for AboutHandler {
    fn call(
        &self,
        _req: Request<Body>,
    ) -> Pin<Box<dyn Future<Output = Result<Response<Body>, hyper::Error>> + Send + Sync>> {
        let message = self.message.clone();
        Box::pin(async { Ok(Response::new(Body::from(message))) })
    }
}
