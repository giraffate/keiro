use crate::Params;
use hyper::{Body, Request};

pub trait RequestExt {
    fn params(&self) -> Option<&Params>;
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
