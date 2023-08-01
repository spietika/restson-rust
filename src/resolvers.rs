use core::task::{Context, Poll};

use hyper::client::connect::dns::{self, GaiResolver as HyperGaiResolver};
use hyper::service::Service;

/// Newtype wrapper around hyper's GaiResolver to provide Default
/// trait implementation
#[derive(Clone, Debug)]
pub struct GaiResolver(HyperGaiResolver);

impl GaiResolver {
    pub fn new() -> Self {
        Self::default()
    }
}

impl From<HyperGaiResolver> for GaiResolver {
    fn from(gai: HyperGaiResolver) -> Self {
        Self(gai)
    }
}

impl Into<HyperGaiResolver> for GaiResolver {
    fn into(self) -> HyperGaiResolver {
        self.0
    }
}

impl Default for GaiResolver {
    fn default() -> Self {
        Self(HyperGaiResolver::new())
    }
}

impl Service<dns::Name> for GaiResolver {
    type Response = <HyperGaiResolver as Service<dns::Name>>::Response;
    type Error = <HyperGaiResolver as Service<dns::Name>>::Error;
    type Future = <HyperGaiResolver as Service<dns::Name>>::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, name: dns::Name) -> Self::Future {
        self.0.call(name)
    }
}
