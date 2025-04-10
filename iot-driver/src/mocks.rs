use std::time::Duration;

use failure::Error;
use futures::Future;
use tokio::{
    task::yield_now,
    time::{sleep, sleep_until, Instant},
};
use tokio_util::sync::CancellationToken;

use crate::{
    Config, Configuration, Connection, ConnectionPool, Datapoint, Fieldbus, FieldbusData, Monitor,
    Subscriber,
};

#[derive(Clone, Debug)]
pub struct MockSubscriber(i32, String, Instant);
impl Subscriber<MockConfig> for MockSubscriber {
    fn new() -> Self {
        Self(0, "1".into(), Instant::now() + Duration::from_secs(2))
    }

    fn subscribe(&self) -> impl Future<Output = Self> {
        async move {
            if self.0 == 0 {
                sleep_until(self.2.into()).await;
                Self(self.0 + 1, self.1.clone(), self.2)
            } else {
                yield_now().await;
                self.clone()
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct MockDatapoint(String, String);
impl Datapoint for MockDatapoint {
    fn new(topic: String) -> Self {
        Self {
            0: topic,
            1: "192.168.20.201:502".into(),
        }
    }

    fn url(&self) -> String {
        self.1.clone()
    }
}

#[derive(Clone, Debug)]
pub struct MockConfiguration(Vec<MockDatapoint>, MockSubscriber, i32);
impl Configuration<MockConfig> for MockConfiguration {
    fn new(subscriber: <MockConfig as Config>::Subscriber) -> Self {
        Self(
            vec![
                MockDatapoint::new("topic/1".into()),
                MockDatapoint::new("topic/2".into()),
            ],
            subscriber,
            0,
        )
    }

    fn subscriber(&self) -> &<MockConfig as Config>::Subscriber {
        &self.1
    }

    fn from_subscriber(&mut self, subscriber: &<MockConfig as Config>::Subscriber) {
        self.1 = subscriber.clone();
        self.2 = subscriber.1.parse().unwrap();
    }

    fn datapoints(&mut self) -> impl Future<Output = MockDatapoint> {
        async move {
            if self.0.len() == 0 {
                self.0 = self.0.clone();
            }
            self.0.rotate_left(1);
            self.0.first().unwrap().clone()
        }
    }
}

#[derive(Clone, Debug)]
pub struct MockData(i32);
impl FieldbusData for MockData {
    fn new() -> Self {
        Self(0)
    }
}

#[derive(Clone, Debug)]
pub struct MockConnection {}
impl Connection for MockConnection {}

#[derive(Clone, Debug)]
pub struct MockConnectionPool(String);
impl ConnectionPool<MockConfig> for MockConnectionPool {
    fn new(url: &str) -> Self {
        Self(url.into())
    }

    fn get(&self) -> impl Future<Output = MockConnection> {
        async { MockConnection {} }
    }
}

#[derive(Clone, Debug)]
pub struct MockFieldbus(i32, Vec<String>);
impl Fieldbus<MockConfig> for MockFieldbus {
    fn new() -> Self {
        Self(0, vec![])
    }

    fn upsert_pool(&mut self, url: String) -> impl Future<Output = Self> {
        async move {
            let mut new = self.1.clone();
            new.push(url);
            Self(self.0.clone(), new)
        }
    }

    fn search_pool(&self, _url: String) -> impl Future<Output = MockConnectionPool> {
        async move { MockConnectionPool(self.1[0].clone()) }
    }

    fn read(&mut self, datapoint: &MockDatapoint) -> impl Future<Output = MockData> {
        async move {
            let mut data = MockData::new();

            if datapoint.0 == "topic/1" {
                data.0 = 1;
            } else if datapoint.0 == "topic/2" {
                data.0 = 2;
            }
            data
        }
    }
}

#[derive(Clone, Debug)]
pub struct MockMonitor(MockConfiguration, CancellationToken, MockFieldbus, MockData);
impl Monitor<MockConfig> for MockMonitor {
    fn new(
        configuration: MockConfiguration,
        cancellation_token: CancellationToken,
        fieldbus: MockFieldbus,
    ) -> Self {
        Self(configuration, cancellation_token, fieldbus, MockData::new())
    }

    fn configuration(&mut self) -> &mut MockConfiguration {
        &mut self.0
    }

    fn cancellation_token(&self) -> &CancellationToken {
        &self.1
    }

    fn fieldbus(&mut self) -> &mut MockFieldbus {
        &mut self.2
    }

    fn from_configuration(
        &mut self,
        cfg: &MockConfiguration,
    ) -> impl Future<Output = Result<(), Error>> {
        async {
            self.0 = cfg.clone();
            Ok(())
        }
    }

    fn process_data(&mut self, d: &MockData) -> impl Future<Output = Result<(), Error>> {
        async {
            sleep(Duration::from_millis(100)).await;
            self.3 = d.clone();
            Ok(())
        }
    }

    fn process_configuration(&mut self) -> impl Future<Output = Result<(), Error>> {
        async { Ok(()) }
    }
}

impl MockMonitor {
    pub fn data(&self) -> i32 {
        self.3 .0
    }

    pub fn config_str(&self) -> i32 {
        self.0 .2
    }
}

#[derive(Clone, Debug)]
pub struct MockConfig {}
impl Config for MockConfig {
    type Subscriber = MockSubscriber;
    type Configuration = MockConfiguration;
    type ConnectionPool = MockConnectionPool;
    type Monitor = MockMonitor;
    type Connection = MockConnection;
    type Fieldbus = MockFieldbus;
    type Datapoint = MockDatapoint;
    type FieldbusData = MockData;
}
