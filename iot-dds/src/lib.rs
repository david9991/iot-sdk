use std::future::Future;

use iot_driver::{Config, Subscriber};

#[derive(Clone)]
pub struct DdsSubscriber {
    domain: u32,
}

impl<T: Config> Subscriber<T> for DdsSubscriber {
    fn new() -> Self {
        Self::from_domain(0)
    }

    fn subscribe(&self) -> impl Future<Output = Self> {
        async move { self.clone() }
    }
}

impl DdsSubscriber {
    fn from_domain(domain: u32) -> Self {
        DdsSubscriber { domain }
    }
}
