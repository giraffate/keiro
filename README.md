# Keiro
Keiro is a lightweight router for Rust HTTP services. It is based on [hyper](https://github.com/hyperium/hyper).

## Install
Add this to `Cargo.toml`:
```toml
[dependencies]
keiro = "0.0.2"
hyper = "0.14"
tokio = { version = "1", features = ["full"] }
```

## Usage
```rust
use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::{Body, Request, Response, Server};
use keiro::prelude::*;
use keiro::Router;

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

async fn index(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Hello keiro!")))
}

async fn hello(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let params = req.params().unwrap();
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
```

### Routing

Keiro uses [`route-recognier`](https://github.com/http-rs/route-recognizer) and supports
four kinds of route segments:
- segments: these are of the format `/a/b`.
- params: these are of the format `/a/:b`.
- named wildcards: these are of the format `/a/*b`.
- unnamed wildcards: these are of the format `/a/*`.

See [here](https://docs.rs/route-recognizer/0.3.0/route_recognizer/#routing-params) for details.

### Middleware

[`tower-http`](https://github.com/tower-rs/tower-http) supports useful middelwares and Keiro can use them.
See [here](/examples/tower_http.rs) for details.

### Not found handler

To handle requests which couldn't be matched by Keiro, `not_found` handler can be used.
See [here](/examles/not_found.rs) for details.

### Share states

Handler can use share states. See [here](/examples/with_state.rs) for details.

## Contributing
1. Fork
2. Create a feature branch
3. Commit your changes
4. Rebase your local changes against the master branch
5. Run test suite with the `cargo test` command and confirm that it passes
6. Run `cargo fmt` and pass `cargo clippy`
7. Create new Pull Request

## License
[MIT license](LICENSE)
