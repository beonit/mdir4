use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
};

use crate::runtime::job::{CancelHandle, JobControl};

use super::control::{RemoteJobOutcome, RemoteRequest, RemoteResult, terminal_outcome};

const DEFAULT_CAPACITY: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteLaneError {
    Busy,
    Closed,
}

type RemoteTaskFn<T> = Box<dyn FnOnce(&JobControl) -> Result<T, RemoteJobOutcome> + Send>;

struct RemoteTask<T> {
    request: RemoteRequest,
    control: JobControl,
    run: RemoteTaskFn<T>,
}

pub struct RemoteLane<T: Send + 'static> {
    requests: Option<mpsc::SyncSender<RemoteTask<T>>>,
    completions: mpsc::Receiver<RemoteResult<T>>,
    active_cancel: Arc<Mutex<Option<CancelHandle>>>,
    shutdown: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl<T: Send + 'static> RemoteLane<T> {
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let (request_sender, request_receiver) =
            mpsc::sync_channel::<RemoteTask<T>>(capacity.max(1));
        let (completion_sender, completion_receiver) =
            mpsc::sync_channel::<RemoteResult<T>>(capacity.max(1));
        let active_cancel = Arc::new(Mutex::new(None));
        let worker_cancel = active_cancel.clone();
        let shutdown = Arc::new(AtomicBool::new(false));
        let worker_shutdown = shutdown.clone();
        let handle = thread::spawn(move || {
            while let Ok(task) = request_receiver.recv() {
                if worker_shutdown.load(Ordering::Acquire) {
                    break;
                }
                let cancel = task.control.cancel_handle();
                *worker_cancel.lock().expect("remote lane lock poisoned") = Some(cancel);
                let result = terminal_outcome(&task.control).map_or_else(
                    || {
                        catch_unwind(AssertUnwindSafe(|| (task.run)(&task.control)))
                            .unwrap_or(Err(RemoteJobOutcome::Failed))
                    },
                    Err,
                );
                let result = terminal_outcome(&task.control).map_or(result, Err);
                *worker_cancel.lock().expect("remote lane lock poisoned") = None;
                if completion_sender
                    .send(RemoteResult {
                        request: task.request,
                        result,
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        Self {
            requests: Some(request_sender),
            completions: completion_receiver,
            active_cancel,
            shutdown,
            handle: Some(handle),
        }
    }

    pub fn try_submit(
        &self,
        request: RemoteRequest,
        control: JobControl,
        run: impl FnOnce(&JobControl) -> Result<T, RemoteJobOutcome> + Send + 'static,
    ) -> Result<(), RemoteLaneError> {
        let Some(requests) = &self.requests else {
            return Err(RemoteLaneError::Closed);
        };
        requests
            .try_send(RemoteTask {
                request,
                control,
                run: Box::new(run),
            })
            .map_err(|error| match error {
                mpsc::TrySendError::Full(_) => RemoteLaneError::Busy,
                mpsc::TrySendError::Disconnected(_) => RemoteLaneError::Closed,
            })
    }

    pub fn cancel_active(&self) {
        if let Some(cancel) = self
            .active_cancel
            .lock()
            .expect("remote lane lock poisoned")
            .as_ref()
        {
            cancel.cancel();
        }
    }

    pub fn try_recv(&self) -> Result<RemoteResult<T>, mpsc::TryRecvError> {
        self.completions.try_recv()
    }
}

impl<T: Send + 'static> Default for RemoteLane<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Send + 'static> Drop for RemoteLane<T> {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Release);
        self.cancel_active();
        self.requests.take();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::mpsc,
        time::{Duration, Instant},
    };

    use crate::{
        remote::{
            control::{SessionEpoch, ViewGeneration},
            location::LocationId,
        },
        runtime::job::Deadline,
    };

    use super::*;

    fn request() -> (CancelHandle, RemoteRequest, JobControl) {
        RemoteRequest::new(
            LocationId::new("dev").unwrap(),
            SessionEpoch(1),
            ViewGeneration(1),
            Deadline::after(Duration::from_secs(1)),
        )
    }

    #[test]
    fn lane_runs_work_and_preserves_the_request_identity() {
        let lane = RemoteLane::new();
        let (_, request, control) = request();
        let expected = request.clone();
        lane.try_submit(request, control, |_| Ok("listed".to_string()))
            .unwrap();

        let completion = loop {
            if let Ok(completion) = lane.try_recv() {
                break completion;
            }
            thread::yield_now();
        };
        assert_eq!(completion.request, expected);
        assert_eq!(completion.result, Ok("listed".to_string()));
    }

    #[test]
    fn active_cancellation_is_outside_the_serial_request_queue() {
        let lane = RemoteLane::new();
        let (_, request, control) = request();
        let (started_sender, started_receiver) = mpsc::channel();
        lane.try_submit(request, control, move |control| {
            started_sender.send(()).unwrap();
            while !control.cancel.is_cancelled() {
                thread::yield_now();
            }
            Ok(())
        })
        .unwrap();
        started_receiver
            .recv_timeout(Duration::from_secs(1))
            .unwrap();
        lane.cancel_active();

        let deadline = Instant::now() + Duration::from_secs(1);
        let completion = loop {
            if let Ok(completion) = lane.try_recv() {
                break completion;
            }
            assert!(Instant::now() < deadline);
            thread::yield_now();
        };
        assert_eq!(completion.result, Err(RemoteJobOutcome::Cancelled));
    }
}
