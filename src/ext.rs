use crate::Params;
use hyper::{Body, Request};

/// An extension trait for [`hyper::Request`](https://docs.rs/hyper/0.14/hyper/struct.Request.html).
pub trait RequestExt {
    /// Get routing parameters.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use std::convert::Infallible;
    /// # use std::net::SocketAddr;
    /// #
    /// # use hyper::{Body, Request, Response, Server};
    /// use keiro::prelude::*;
    /// # use keiro::Router;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let mut router = Router::new();
    ///     router.get("/hello/:user1/from/:user2", hello);
    ///     let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    ///
    ///     Server::bind(&addr)
    ///         .serve(router.into_service())
    ///         .await
    ///         .unwrap();
    /// }
    ///
    /// async fn hello(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    ///     let params = req.params().unwrap();
    ///     Ok(Response::new(Body::from(format!(
    ///         "Hello {} from {}!",
    ///         params.find("user1").unwrap(),
    ///         params.find("user2").unwrap(),
    ///     ))))
    /// }
    /// ```
    fn params(&self) -> Option<&Params>;

    /// Get shared states.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use std::convert::Infallible;
    /// # use std::net::SocketAddr;
    /// #
    /// # use hyper::{Body, Request, Response, Server};
    /// use keiro::prelude::*;
    /// # use keiro::Router;
    ///
    /// #[derive(Clone)]
    /// struct State {
    ///     name: String,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let state = State {
    ///         name: "giraffate".to_string(),
    ///     };
    ///     let mut router = Router::with_state(state);
    ///     router.get("/", index);
    ///     let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    ///
    ///     Server::bind(&addr)
    ///         .serve(router.into_service())
    ///         .await
    ///         .unwrap();
    /// }
    ///
    /// async fn index(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    ///     let state = req.state::<State>().unwrap();
    ///     Ok(Response::new(Body::from(format!("Hello {}", state.name))))
    /// }
    /// ```
    fn state<T: Clone + Send + Sync + 'static>(&self) -> Option<&T>;
}

impl RequestExt for Request<Body> {
    fn params(&self) -> Option<&Params> {
        self.extensions().get::<Params>()
    }

    fn state<T: Clone + Send + Sync + 'static>(&self) -> Option<&T> {
        self.extensions().get::<T>()
    }
}
