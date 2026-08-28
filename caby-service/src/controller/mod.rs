use std::{
    any::Any,
    panic::{self, AssertUnwindSafe},
    sync::{Arc, Mutex},
};

use anyhow::anyhow;
use futures_util::FutureExt;
use jiff::SignedDuration;
use tokio::{
    sync::{Notify, OwnedSemaphorePermit, Semaphore},
    task, time,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{debug, warn};

use crate::{
    config::Config,
    event::{self, Event, Receiver, Sender},
    job::{Input, Job},
    Result,
};

pub mod jobs;
pub mod registry;

pub use crate::job::Priority;

pub type EventHandler = fn(&Event) -> Vec<(Priority, Input)>;

use registry::{Pending, Registry, Released};

// todo: move these to controller settings in config
const MAX_CONCURRENT_JOBS: usize = 4;
const MAX_ATTEMPTS: u32 = 8;
const RETRY_BASE: SignedDuration = SignedDuration::from_secs(2);
const RETRY_CEILING: SignedDuration = SignedDuration::from_secs(300);
const RETRY_MAX_EXPONENT: u32 = 16;

fn retry_backoff(attempts: u32) -> SignedDuration {
    let factor = 1i32 << attempts.saturating_sub(1).min(RETRY_MAX_EXPONENT);

    RETRY_BASE
        .checked_mul(factor)
        .unwrap_or(RETRY_CEILING)
        .min(RETRY_CEILING)
}

// todo: move housekeeping into controller
// todo: add controller settings to config
pub struct Controller {
    cfg: Config,
    handlers: Vec<EventHandler>,
    registry: Mutex<Registry>,
    // todo: notification system that we can wire to websockets
    // used to signal the job queue to start, today
    notify: Notify,
    slots: Arc<Semaphore>,
    tasks: TaskTracker,
    cancel: CancellationToken,
}

fn panic_message(panic: &(dyn Any + Send)) -> String {
    if let Some(message) = panic.downcast_ref::<&str>() {
        return (*message).to_string();
    }

    if let Some(message) = panic.downcast_ref::<String>() {
        return message.clone();
    }

    "unknown panic".to_string()
}

pub struct JobLease {
    controller: Arc<Controller>,
    job: Job,
    requeue: bool,
}

impl JobLease {
    fn new(controller: Arc<Controller>, job: Job) -> Self {
        Self {
            controller,
            job,
            requeue: true,
        }
    }

    pub fn job(&self) -> &Job {
        &self.job
    }

    fn settle(&mut self, result: &Result<()>) {
        self.requeue = result.is_err();
    }
}

impl Drop for JobLease {
    fn drop(&mut self) {
        self.controller.release_job(&self.job.id, self.requeue);
    }
}

impl Controller {
    pub fn new(cfg: Config) -> (Arc<Self>, Sender) {
        let (events_tx, events_rx) = event::channel();

        let controller = Arc::new(Self {
            cfg,
            handlers: jobs::handlers(),
            registry: Mutex::new(Registry::default()),
            notify: Notify::new(),
            slots: Arc::new(Semaphore::new(MAX_CONCURRENT_JOBS)),
            tasks: TaskTracker::new(),
            cancel: CancellationToken::new(),
        });

        controller.start(events_rx);
        (controller, events_tx)
    }

    fn start(self: &Arc<Self>, events_rx: Receiver) {
        task::spawn(self.clone().run_jobs());
        task::spawn(self.clone().run_events(events_rx));
    }

    pub async fn shutdown(&self) {
        self.cancel.cancel();
        self.notify.notify_waiters();
        self.tasks.close();
        self.tasks.wait().await;
    }

    // Jobs

    async fn supervise_job(self: Arc<Self>, mut lease: JobLease, _slot: OwnedSemaphorePermit) {
        let result = AssertUnwindSafe(jobs::run(&self.cfg, &lease.job().input))
            .catch_unwind()
            .await
            .unwrap_or_else(|panic| Err(anyhow!("job panicked: {}", panic_message(&*panic))));

        if let Err(err) = &result {
            warn!("controller: job {:?} failed: {:#}", lease.job().input, err);
        }

        lease.settle(&result);
    }

    async fn next_job(self: &Arc<Self>) -> Option<JobLease> {
        loop {
            let notified = self.notify.notified();

            let pending = self
                .registry
                .lock()
                .unwrap_or_else(|err| err.into_inner())
                .take_pending();

            match pending {
                Pending::Ready(job) => return Some(JobLease::new(self.clone(), job)),
                Pending::Backoff(delay) => {
                    tokio::select! {
                        _ = self.cancel.cancelled() => return None,
                        _ = time::sleep(delay) => {}
                        _ = notified => {}
                    }
                }
                Pending::Empty => {
                    tokio::select! {
                        _ = self.cancel.cancelled() => return None,
                        _ = notified => {}
                    }
                }
            }
        }
    }

    fn release_job(&self, id: &str, requeue: bool) {
        let released = {
            let mut registry = self.registry.lock().unwrap_or_else(|err| err.into_inner());
            match requeue {
                true => registry.requeue(id),
                false => {
                    registry.complete(id);
                    Released::Done
                }
            }
        };

        match released {
            Released::Retrying(delay) => debug!(
                job.id = id,
                retry_in_ms = delay.as_millis() as u64,
                "job requeued"
            ),
            Released::GaveUp => warn!(
                job.id = id,
                attempts = MAX_ATTEMPTS as u64,
                "job exhausted its attempts, dropping"
            ),
            Released::Done => {}
        }

        self.notify.notify_one();
    }

    async fn run_jobs(self: Arc<Self>) {
        loop {
            let slot = tokio::select! {
                _ = self.cancel.cancelled() => break,
                slot = self.slots.clone().acquire_owned() => match slot {
                    Ok(slot) => slot,
                    Err(_) => break,
                },
            };

            let Some(lease) = self.next_job().await else {
                break;
            };

            self.tasks.spawn(self.clone().supervise_job(lease, slot));
        }

        self.tasks.close();
    }

    pub fn schedule_job(&self, priority: Priority, input: Input) -> bool {
        let queued = self
            .registry
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .insert(input, priority);

        if queued {
            self.notify.notify_one();
        }

        queued
    }

    // Events

    fn dispatch_event(&self, event: Event) {
        for handler in &self.handlers {
            let inputs = match panic::catch_unwind(AssertUnwindSafe(|| handler(&event))) {
                Ok(inputs) => inputs,
                Err(panic) => {
                    let message = panic_message(&*panic);
                    warn!(error = message.as_str(), "event handler panicked");
                    continue;
                }
            };

            for (priority, input) in inputs {
                self.schedule_job(priority, input);
            }
        }
    }

    async fn run_events(self: Arc<Self>, mut events_rx: Receiver) {
        loop {
            tokio::select! {
                _ = self.cancel.cancelled() => break,
                event = events_rx.recv() => match event {
                    Some(event) => self.dispatch_event(event),
                    None => break,
                },
            }
        }
    }
}
