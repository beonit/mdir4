use std::{
    collections::{HashMap, VecDeque},
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, Condvar, Mutex, mpsc},
    thread,
};

use crate::runtime::job::{CancelHandle, JobControl};

use super::{
    api::{PluginEffect, PluginError, PluginId, PluginPayload, PluginResult},
    manager::{PluginDispatch, PluginManager},
};

pub const DEFAULT_PLUGIN_READ_CAPACITY: usize = 16;

pub type PluginReadJob = Box<dyn FnOnce(JobControl) -> Result<PluginPayload, PluginError> + Send>;

pub struct PluginReadRequest {
    pub effect: PluginEffect,
    pub control: JobControl,
    pub job: PluginReadJob,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginReadSubmit {
    Queued,
    Coalesced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginReadBusy {
    Busy,
    Closed,
    Inactive,
}

struct QueueState {
    closing: bool,
    queued: VecDeque<PluginReadRequest>,
    active: HashMap<PluginId, CancelHandle>,
}

pub struct PluginReadLane {
    state: Arc<(Mutex<QueueState>, Condvar)>,
    completions: mpsc::Receiver<PluginResult>,
    handle: Option<thread::JoinHandle<()>>,
    capacity: usize,
}

impl PluginReadLane {
    pub fn spawn(capacity: usize) -> Self {
        assert!(capacity > 0, "plugin read capacity must be positive");
        let state = Arc::new((
            Mutex::new(QueueState {
                closing: false,
                queued: VecDeque::new(),
                active: HashMap::new(),
            }),
            Condvar::new(),
        ));
        let (completion_sender, completions) = mpsc::channel();
        let worker_state = Arc::clone(&state);
        let handle = thread::spawn(move || worker_loop(worker_state, completion_sender));
        Self {
            state,
            completions,
            handle: Some(handle),
            capacity,
        }
    }

    pub fn submit(&self, request: PluginReadRequest) -> Result<PluginReadSubmit, PluginReadBusy> {
        let (lock, wake) = &*self.state;
        let mut state = lock.try_lock().map_err(|_| PluginReadBusy::Busy)?;
        if state.closing {
            return Err(PluginReadBusy::Closed);
        }
        if let Some(existing) = state
            .queued
            .iter_mut()
            .find(|queued| queued.effect.plugin_id == request.effect.plugin_id)
        {
            *existing = request;
            wake.notify_one();
            return Ok(PluginReadSubmit::Coalesced);
        }
        if state.queued.len() >= self.capacity {
            return Err(PluginReadBusy::Busy);
        }
        state.queued.push_back(request);
        wake.notify_one();
        Ok(PluginReadSubmit::Queued)
    }

    pub fn submit_for_active(
        &self,
        manager: &PluginManager,
        request: PluginReadRequest,
    ) -> Result<PluginReadSubmit, PluginReadBusy> {
        if !manager.accepts_effect(&request.effect) {
            return Err(PluginReadBusy::Inactive);
        }
        self.submit(request)
    }

    pub fn cancel(&self, plugin_id: &PluginId) {
        let (lock, _) = &*self.state;
        if let Ok(state) = lock.try_lock()
            && let Some(handle) = state.active.get(plugin_id)
        {
            handle.cancel();
        }
    }

    pub fn try_result(&self) -> Option<PluginResult> {
        self.completions.try_recv().ok()
    }

    pub fn drain_into_manager(&self, manager: &mut PluginManager) -> Vec<PluginDispatch> {
        let mut dispatches = Vec::new();
        while let Some(result) = self.try_result() {
            dispatches.push(manager.handle_result(result));
        }
        dispatches
    }

    pub fn shutdown(&mut self) {
        let (lock, wake) = &*self.state;
        if let Ok(mut state) = lock.lock() {
            state.closing = true;
            state.queued.clear();
            for handle in state.active.values() {
                handle.cancel();
            }
            wake.notify_all();
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for PluginReadLane {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn worker_loop(state: Arc<(Mutex<QueueState>, Condvar)>, completions: mpsc::Sender<PluginResult>) {
    loop {
        let request = {
            let (lock, wake) = &*state;
            let mut guard = lock.lock().expect("plugin read queue mutex poisoned");
            while guard.queued.is_empty() && !guard.closing {
                guard = wake.wait(guard).expect("plugin read queue mutex poisoned");
            }
            if guard.closing {
                return;
            }
            let request = guard.queued.pop_front().expect("non-empty plugin queue");
            guard.active.insert(
                request.effect.plugin_id.clone(),
                request.control.cancel_handle(),
            );
            request
        };

        let effect = request.effect.clone();
        let control = request.control.clone();
        let outcome = match catch_unwind(AssertUnwindSafe(|| (request.job)(control.clone()))) {
            Ok(_outcome) if control.cancel.is_cancelled() => {
                Err(PluginError::new("plugin read cancelled"))
            }
            Ok(_outcome) if control.deadline.expired() => {
                Err(PluginError::new("plugin read timed out"))
            }
            Ok(outcome) => outcome,
            Err(_) => Err(PluginError::new("plugin read job panicked")),
        };
        let result = PluginResult {
            plugin_id: effect.plugin_id.clone(),
            generation: effect.generation,
            request_id: effect.request_id,
            outcome,
        };
        let (lock, _) = &*state;
        if let Ok(mut guard) = lock.lock() {
            guard.active.remove(&effect.plugin_id);
        }
        if completions.send(result).is_err() {
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Barrier},
        time::{Duration, Instant},
    };

    use super::*;
    use crate::{
        plugins::api::{PluginEffectKind, PluginGeneration, PluginRequestId},
        runtime::job::Deadline,
    };

    fn request(id: &str, generation: u64, request: u64, job: PluginReadJob) -> PluginReadRequest {
        let plugin_id = PluginId::new(id).unwrap();
        let (_handle, control) = JobControl::new(Deadline::after(Duration::from_secs(2)));
        PluginReadRequest {
            effect: PluginEffect {
                plugin_id,
                generation: PluginGeneration(generation),
                request_id: PluginRequestId(request),
                kind: PluginEffectKind::Refresh,
            },
            control,
            job,
        }
    }

    #[test]
    fn capacity_is_non_blocking_and_queued_refresh_is_coalesced_to_the_latest_request() {
        let lane = PluginReadLane::spawn(1);
        let gate = Arc::new(Barrier::new(2));
        let (started_sender, started_receiver) = mpsc::sync_channel(1);
        let running_gate = Arc::clone(&gate);
        assert_eq!(
            lane.submit(request(
                "one",
                1,
                1,
                Box::new(move |_| {
                    started_sender.send(()).unwrap();
                    running_gate.wait();
                    Ok(PluginPayload::new(PluginId::new("one").unwrap(), "running"))
                })
            )),
            Ok(PluginReadSubmit::Queued)
        );
        started_receiver
            .recv_timeout(Duration::from_secs(1))
            .unwrap();
        let start = Instant::now();
        assert_eq!(
            lane.submit(request(
                "two",
                1,
                1,
                Box::new(|_| Ok(PluginPayload::new(PluginId::new("two").unwrap(), "old")))
            )),
            Ok(PluginReadSubmit::Queued)
        );
        assert_eq!(
            lane.submit(request(
                "two",
                2,
                2,
                Box::new(|_| Ok(PluginPayload::new(PluginId::new("two").unwrap(), "new")))
            )),
            Ok(PluginReadSubmit::Coalesced)
        );
        assert_eq!(
            lane.submit(request(
                "three",
                1,
                1,
                Box::new(|_| Ok(PluginPayload::new(PluginId::new("three").unwrap(), "busy")))
            )),
            Err(PluginReadBusy::Busy)
        );
        assert!(start.elapsed() < Duration::from_millis(50));
        gate.wait();
        let first = lane
            .completions
            .recv_timeout(Duration::from_secs(1))
            .unwrap();
        let second = lane
            .completions
            .recv_timeout(Duration::from_secs(1))
            .unwrap();
        assert_eq!(first.request_id, PluginRequestId(1));
        assert_eq!(second.plugin_id, PluginId::new("two").unwrap());
        assert_eq!(second.generation, PluginGeneration(2));
    }

    #[test]
    fn cancellation_panic_and_shutdown_produce_a_terminal_result_and_join() {
        let mut lane = PluginReadLane::spawn(1);
        let gate = Arc::new(Barrier::new(2));
        let (started_sender, started_receiver) = mpsc::sync_channel(1);
        let running_gate = Arc::clone(&gate);
        lane.submit(request(
            "cancel",
            1,
            1,
            Box::new(move |control| {
                started_sender.send(()).unwrap();
                running_gate.wait();
                while !control.cancel.is_cancelled() {}
                Ok(PluginPayload::new(PluginId::new("cancel").unwrap(), ()))
            }),
        ))
        .unwrap();
        started_receiver
            .recv_timeout(Duration::from_secs(1))
            .unwrap();
        lane.cancel(&PluginId::new("cancel").unwrap());
        gate.wait();
        let cancelled = lane
            .completions
            .recv_timeout(Duration::from_secs(1))
            .unwrap();
        assert_eq!(
            cancelled.outcome.unwrap_err().message(),
            "plugin read cancelled"
        );

        lane.submit(request("panic", 1, 2, Box::new(|_| panic!("test panic"))))
            .unwrap();
        let panicked = lane
            .completions
            .recv_timeout(Duration::from_secs(1))
            .unwrap();
        assert_eq!(
            panicked.outcome.unwrap_err().message(),
            "plugin read job panicked"
        );
        lane.shutdown();
        assert_eq!(
            lane.submit(request("closed", 1, 3, Box::new(|_| unreachable!()))),
            Err(PluginReadBusy::Closed)
        );
    }
}
