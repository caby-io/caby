// Much of this is inspired and taken from Kubernetes' jobs workqueue system:
// https://pkg.go.dev/k8s.io/client-go/util/workqueue

use std::{collections::HashMap, path::PathBuf, time::Duration};

use jiff::Timestamp;

use crate::job::{Input, Job, LockKey, Priority, Status};

use super::{retry_backoff, MAX_ATTEMPTS};

pub enum Pending {
    Ready(Job),
    Backoff(Duration),
    Empty,
}

pub enum Released {
    Done,
    Retrying(Duration),
    GaveUp,
}

#[derive(Default)]
pub struct Registry {
    jobs: HashMap<String, Job>,
    // indicies to help with lookup
    // todo: lookup by dir or file
    // todo: lookup by job type
}

fn until(at: Timestamp, now: Timestamp) -> Duration {
    Duration::from_millis(at.duration_since(now).as_millis().max(0) as u64)
}

impl Registry {
    pub fn insert(&mut self, input: Input, priority: Priority) -> bool {
        if let Some(pending) = self
            .jobs
            .values_mut()
            .find(|job| job.status == Status::Pending && job.input == input)
        {
            if priority == Priority::Interactive {
                pending.priority = priority;
                pending.not_before = None;
            }
            return false;
        }

        let job = Job::new(input, priority);
        self.jobs.insert(job.id.clone(), job);
        true
    }

    pub fn take_pending(&mut self) -> Pending {
        let now = Timestamp::now();

        let held: Vec<LockKey> = self
            .jobs
            .values()
            .filter(|job| job.status == Status::Running)
            .flat_map(|job| job.input.locks())
            .collect();

        let mut next: Option<&Job> = None;
        let mut backoff: Option<Timestamp> = None;

        for job in self.jobs.values() {
            if job.status != Status::Pending {
                continue;
            }

            if job
                .input
                .locks()
                .iter()
                .any(|key| held.iter().any(|other| key.conflicts(other)))
            {
                continue;
            }

            if !job.is_ready(now) {
                let at = job.not_before.expect("not ready implies not_before");
                backoff = Some(backoff.map_or(at, |soonest: Timestamp| soonest.min(at)));
                continue;
            }

            if job.priority == Priority::Interactive {
                next = Some(job);
                break;
            }

            if next.is_none() {
                next = Some(job);
            }
        }

        let Some(id) = next.map(|job| job.id.clone()) else {
            return match backoff {
                Some(at) => Pending::Backoff(until(at, now)),
                None => Pending::Empty,
            };
        };

        let Some(job) = self.jobs.get_mut(&id) else {
            return Pending::Empty;
        };
        job.status = Status::Running;

        Pending::Ready(job.clone())
    }

    pub fn complete(&mut self, id: &str) {
        self.jobs.remove(id);
    }

    pub fn requeue(&mut self, id: &str) -> Released {
        let Some(job) = self.jobs.get_mut(id) else {
            return Released::Done;
        };

        job.attempts += 1;
        if job.attempts >= MAX_ATTEMPTS {
            self.jobs.remove(id);
            return Released::GaveUp;
        }

        let delay = retry_backoff(job.attempts);
        let at = Timestamp::now() + delay;

        job.status = Status::Pending;
        job.not_before = Some(at);

        Released::Retrying(Duration::from_millis(delay.as_millis().max(0) as u64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reconcile(path: &str) -> Input {
        Input::ReconcileShare {
            space: "home".to_owned(),
            path: PathBuf::from(path),
        }
    }

    fn take(registry: &mut Registry) -> Job {
        match registry.take_pending() {
            Pending::Ready(job) => job,
            Pending::Backoff(delay) => panic!("expected a ready job, backing off {delay:?}"),
            Pending::Empty => panic!("expected a ready job, registry empty"),
        }
    }

    #[test]
    fn insert_dedups_pending_and_promotes_priority() {
        let mut registry = Registry::default();

        assert!(registry.insert(reconcile("a.caby.yaml"), Priority::Background));
        assert!(!registry.insert(reconcile("a.caby.yaml"), Priority::Interactive));

        let job = take(&mut registry);
        assert_eq!(job.priority, Priority::Interactive);
    }

    #[test]
    fn running_input_blocks_an_equal_pending_input() {
        let mut registry = Registry::default();

        registry.insert(reconcile("a.caby.yaml"), Priority::Background);
        let running = take(&mut registry);

        registry.insert(reconcile("a.caby.yaml"), Priority::Background);
        assert!(matches!(registry.take_pending(), Pending::Empty));

        registry.complete(&running.id);
        let job = take(&mut registry);
        assert_eq!(job.input, reconcile("a.caby.yaml"));
    }

    #[test]
    fn different_paths_run_concurrently() {
        let mut registry = Registry::default();

        registry.insert(reconcile("a.caby.yaml"), Priority::Background);
        registry.insert(reconcile("b.caby.yaml"), Priority::Background);

        let first = take(&mut registry);
        let second = take(&mut registry);
        assert_ne!(first.input, second.input);
    }

    #[test]
    fn requeue_backs_off_then_becomes_ready() {
        let mut registry = Registry::default();

        registry.insert(reconcile("a.caby.yaml"), Priority::Background);
        let job = take(&mut registry);

        assert!(matches!(registry.requeue(&job.id), Released::Retrying(_)));

        match registry.take_pending() {
            Pending::Backoff(delay) => assert!(delay > Duration::ZERO),
            _ => panic!("expected the retry to be held back"),
        }

        let requeued = registry.jobs.get_mut(&job.id).unwrap();
        assert_eq!(requeued.attempts, 1);
        requeued.not_before = None;

        let ready = take(&mut registry);
        assert_eq!(ready.attempts, 1);
    }

    #[test]
    fn requeue_gives_up_after_max_attempts() {
        let mut registry = Registry::default();

        registry.insert(reconcile("a.caby.yaml"), Priority::Background);

        for attempt in 1..MAX_ATTEMPTS {
            let job = take(&mut registry);
            assert!(matches!(registry.requeue(&job.id), Released::Retrying(_)));
            registry.jobs.get_mut(&job.id).unwrap().not_before = None;
            assert_eq!(registry.jobs[&job.id].attempts, attempt);
        }

        let job = take(&mut registry);
        assert!(matches!(registry.requeue(&job.id), Released::GaveUp));
        assert!(matches!(registry.take_pending(), Pending::Empty));
    }

    #[test]
    fn interactive_insert_clears_pending_backoff() {
        let mut registry = Registry::default();

        registry.insert(reconcile("a.caby.yaml"), Priority::Background);
        let job = take(&mut registry);
        registry.requeue(&job.id);

        assert!(matches!(registry.take_pending(), Pending::Backoff(_)));

        assert!(!registry.insert(reconcile("a.caby.yaml"), Priority::Interactive));
        let ready = take(&mut registry);
        assert_eq!(ready.priority, Priority::Interactive);
    }

    #[test]
    fn backoff_grows_and_is_capped() {
        assert!(retry_backoff(1) < retry_backoff(2));
        assert!(retry_backoff(2) < retry_backoff(3));
        assert_eq!(retry_backoff(64), retry_backoff(128));
    }
}
