use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use futures::executor::block_on;
use tokio::{sync::RwLock, task::JoinSet, time::sleep_until};
use tokio_util::sync::CancellationToken;

use crate::{
    mocks::{MockConfiguration, MockFieldbus, MockMonitor, MockSubscriber},
    Configuration, Fieldbus, Monitor, Subscriber,
};

#[derive(std::fmt::Debug, PartialEq, Clone)]
struct State(i32);

#[tokio::test]
async fn test_monitor() {
    let subscriber = MockSubscriber::new();
    let configuration = MockConfiguration::new(subscriber);
    let cancellation_token = CancellationToken::new();
    let fieldbus = MockFieldbus::new();
    let mut monitor = MockMonitor::new(configuration, cancellation_token.clone(), fieldbus);
    let state = Arc::new(RwLock::new(State(0)));
    let statew = state.clone();
    let deadline = Instant::now() + Duration::from_secs(10);

    let mut js = JoinSet::new();

    js.spawn(async move {
        monitor
            .run(Some(|mstate: &MockMonitor| {
                block_on(async {
                    *statew.write().await =
                        if *statew.read().await == State(0) && mstate.config_str() == 0 {
                            State(1)
                        } else if *statew.read().await == State(1) && mstate.data() == 1 {
                            State(2)
                        } else if *statew.read().await == State(2) && mstate.data() == 2 {
                            State(3)
                        } else if *statew.read().await == State(3) && mstate.config_str() == 1 {
                            State(4)
                        } else {
                            statew.read().await.clone()
                        };
                    println!("state = {:?}, monitor = {:?}", statew.read().await, mstate);
                });
            }))
            .await
            .unwrap();
    });
    js.spawn(async move {
        sleep_until(deadline.into()).await;
        cancellation_token.cancel();
    });

    js.join_all().await;
    assert_eq!(*state.read().await, State(4));
}
