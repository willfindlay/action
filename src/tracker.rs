use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;

const WINDOW_SIZE: usize = 60;

#[derive(Debug)]
struct TrackerInner {
    actions: [u64; WINDOW_SIZE],
    rolling_count: u64,
    tick: usize,
}

impl Default for TrackerInner {
    fn default() -> Self {
        Self {
            actions: [0u64; WINDOW_SIZE],
            rolling_count: 0,
            tick: Default::default(),
        }
    }
}

impl TrackerInner {}

#[derive(Default, Debug)]
pub struct Tracker {
    inner: Arc<Mutex<TrackerInner>>,
}

impl Tracker {
    pub async fn add(&self) {
        let mut inner = self.inner.lock().await;
        let tick = inner.tick;
        inner.actions[tick % WINDOW_SIZE] += 1;
        tracing::trace!(total = inner.actions[tick % WINDOW_SIZE], "add");
    }

    pub async fn increment_tick(&self) {
        let mut inner = self.inner.lock().await;

        let to_add = inner.actions[inner.tick % WINDOW_SIZE];
        inner.rolling_count += to_add;

        let mut to_sub = None;
        if inner.tick >= WINDOW_SIZE {
            to_sub = Some(inner.actions[(inner.tick - WINDOW_SIZE + 1) % WINDOW_SIZE]);
            inner.rolling_count -= to_sub.unwrap();
        }

        tracing::debug!(
            inner.tick,
            to_add,
            to_sub = to_sub.unwrap_or(0),
            inner.rolling_count,
            "increment_tick"
        );

        inner.tick += 1;
        let tick = inner.tick;
        inner.actions[tick % WINDOW_SIZE] = 0;
    }

    pub async fn apm(&self) -> u64 {
        let inner = self.inner.lock().await;
        if inner.tick == 0 {
            return 0;
        }
        if inner.tick < 60 {
            let m = WINDOW_SIZE as f64 / inner.tick as f64;
            return (m * inner.rolling_count as f64).round() as u64;
        }
        inner.rolling_count
        // let mut acc = 0;
        // for v in inner.actions {
        //     acc += v;
        // }
        // if inner.tick > 0 && inner.tick < WINDOW_SIZE {
        //     let m = WINDOW_SIZE / inner.tick;
        //     acc = acc * m as u64;
        // }
        // acc
    }

    pub async fn track(self: Arc<Self>, rchan: Arc<Mutex<UnboundedReceiver<()>>>) -> Result<()> {
        let rchan = rchan.clone();

        tokio::spawn({
            let tracker = self.clone();
            async move {
                loop {
                    let Some(_) = rchan.lock().await.recv().await else {
                        tracing::error!("received null event through channel");
                        continue;
                    };
                    tracker.add().await;
                }
            }
        });
        tokio::spawn({
            let tracker = self.clone();
            async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    tracker.increment_tick().await;
                }
            }
        });
        Ok(())
    }
}
