use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::Mutex;

const WINDOW_SIZE: usize = 60;

#[derive(Debug)]
struct TrackerInner {
    actions: [u64; WINDOW_SIZE],
    tick: usize,
}

impl Default for TrackerInner {
    fn default() -> Self {
        Self {
            actions: [0u64; WINDOW_SIZE],
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
        inner.tick += 1;
        let tick = inner.tick;
        inner.actions[tick % WINDOW_SIZE] = 0;
        tracing::debug!(new = tick, "increment_tick");
    }

    pub async fn apm(&self) -> u64 {
        let inner = self.inner.lock().await;
        let mut acc = 0;
        for v in inner.actions {
            acc += v;
        }
        if inner.tick > 0 && inner.tick < WINDOW_SIZE {
            let m = WINDOW_SIZE / inner.tick;
            acc = acc * m as u64;
        }
        acc
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
