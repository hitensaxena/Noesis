use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;
use tracing;

use crate::kernel::bus::EventBus;

struct ScheduledTask {
    name: String,
    interval_secs: u64,
}

/// A simple scheduler that runs periodic tasks.
pub struct Scheduler {
    tasks: Vec<ScheduledTask>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    /// Register a periodic task.
    pub fn add_task(&mut self, name: &str, interval_secs: u64) {
        self.tasks.push(ScheduledTask {
            name: name.to_string(),
            interval_secs,
        });
    }

    /// Start all scheduled tasks as tokio tasks.
    /// Returns join handles for the spawned tasks.
    pub fn run_tasks(
        &self,
        _event_bus: Arc<EventBus>,
        cancellation_token: CancellationToken,
    ) -> Vec<tokio::task::JoinHandle<()>> {
        self.tasks
            .iter()
            .map(|t| {
                let name = t.name.clone();
                let interval_dur = Duration::from_secs(t.interval_secs);
                let token = cancellation_token.clone();

                tokio::spawn(async move {
                    let mut timer = interval(interval_dur);
                    // Skip the first immediate tick
                    timer.tick().await;
                    loop {
                        tokio::select! {
                            _ = timer.tick() => {
                                tracing::debug!("[Scheduler] running task: {}", name);
                            }
                            _ = token.cancelled() => {
                                tracing::debug!("[Scheduler] task '{}' cancelled", name);
                                break;
                            }
                        }
                    }
                })
            })
            .collect()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}
