#[cfg(test)]
mod mocks;

#[cfg(test)]
mod tests;

use std::future::Future;

use async_stream::stream;
use failure::Error;
use futures::{pin_mut, Stream, StreamExt};
use tokio::select;
use tokio_util::sync::CancellationToken;

pub trait Subscriber<T: Config>: Clone {
    fn new() -> Self;

    fn subscribe(&self) -> impl Future<Output = Self>;
}

pub trait Datapoint {
    fn new(topic: String) -> Self;
    fn url(&self) -> String;
}

pub trait FieldbusData {
    fn new() -> Self;
}

pub trait Configuration<T: Config>: Clone + From<Self> {
    fn new(subscriber: T::Subscriber) -> Self;
    fn subscriber(&self) -> &T::Subscriber;
    fn from_subscriber(&mut self, subscriber: &T::Subscriber);
    fn datapoints(&mut self) -> impl Future<Output = T::Datapoint>;

    fn subscribe(&mut self) -> impl Future<Output = ()> {
        async {
            self.from_subscriber(&self.subscriber().subscribe().await);
        }
    }
}

pub trait Monitor<T: Config>: Clone {
    fn new(
        configuration: T::Configuration,
        cancellation_token: CancellationToken,
        fieldbus: T::Fieldbus,
    ) -> Self;
    fn configuration(&mut self) -> &mut T::Configuration;
    fn cancellation_token(&self) -> &CancellationToken;
    fn fieldbus(&mut self) -> &mut T::Fieldbus;

    fn poll(&mut self) -> impl Stream<Item = T::FieldbusData> {
        stream! {
            loop {
                let datapoint = self.configuration().datapoints().await;
                yield self.fieldbus().read(&datapoint).await
            }
        }
    }
    fn from_configuration(
        &mut self,
        cfg: &T::Configuration,
    ) -> impl Future<Output = Result<(), Error>>;
    fn process_data(&mut self, d: &T::FieldbusData) -> impl Future<Output = Result<(), Error>>;
    fn process_configuration(&mut self) -> impl Future<Output = Result<(), Error>>;
    fn run<F>(&mut self, state_callback: Option<F>) -> impl Future<Output = Result<(), Error>>
    where
        F: Fn(&Self),
    {
        async move {
            loop {
                let mut configuration = self.configuration().clone();
                let cancellation_token = self.cancellation_token().clone();
                select! {
                    () = configuration.subscribe() => {
                        self.from_configuration(&configuration).await?;
                    },
                    d = async { let poll = self.poll(); pin_mut!(poll); poll.next().await } => {
                        match d {
                            Some(d) => { self.process_data(&d).await?; },
                            None=> ()
                        };
                    }
                    _ = cancellation_token.cancelled() => {
                        break;
                    }
                };
                if let Some(cb) = &state_callback {
                    cb(self);
                }
            }
            Ok(())
        }
    }
}

pub trait Fieldbus<T: Config>: Clone {
    fn new() -> Self;

    fn upsert_pool(&mut self, url: String) -> impl Future<Output = Self>;
    fn search_pool(&self, url: String) -> impl Future<Output = T::ConnectionPool>;
    fn read(&mut self, datapoint: &T::Datapoint) -> impl Future<Output = T::FieldbusData>;
}

pub trait Connection {}

pub trait ConnectionPool<T: Config> {
    fn new(url: &str) -> Self;

    fn get(&self) -> impl Future<Output = T::Connection>;
}

pub trait Config: Clone {
    type Subscriber: Subscriber<Self>;
    type Configuration: Configuration<Self>;
    type ConnectionPool: ConnectionPool<Self>;
    type Connection: Connection;
    type Monitor: Monitor<Self>;
    type Fieldbus: Fieldbus<Self>;
    type Datapoint: Datapoint;
    type FieldbusData: FieldbusData;
}

pub trait GenericDriver<T: Config> {
    fn run() -> impl std::future::Future<Output = ()> {
        async move {
            let subscriber = T::Subscriber::new();
            let configuration = T::Configuration::new(subscriber);
            let cancellation_token = CancellationToken::new();
            let fieldbus = T::Fieldbus::new();
            let mut monitor = T::Monitor::new(configuration, cancellation_token, fieldbus);
            monitor.run::<fn(&T::Monitor)>(None).await.unwrap();
        }
    }
}
