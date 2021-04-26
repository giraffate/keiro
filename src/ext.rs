use crate::Params;
use hyper::{Body, Request};

pub trait RequestExt {
    fn params(&self) -> Option<&Params>;
}

impl RequestExt for Request<Body> {
    fn params(&self) -> Option<&Params> {
        self.extensions().get::<Params>()
    }
}
