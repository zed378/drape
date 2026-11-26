//! Bounded, single-threaded STA dispatch for the synchronous shell icon probe.
//!
//! This returns raw DIB bytes. Channel order and alpha semantics remain unverified,
//! so callers must not reinterpret the response as a Core `IconBitmap` or UI image.

use crate::shell_items::{ShellIconDib, probe_shell_icon};
use drape_core::domain::Guid;
use drape_core::ports::{Cancellation, OperationId, PortError};
use std::ffi::{OsStr, OsString};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::thread::{self, JoinHandle};

const MAX_QUEUED_ICON_REQUESTS: usize = 32;

/// A bounded request for one already-enumerated filesystem item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellIconRequest {
    pub operation: OperationId,
    pub item: Guid,
    pub path: OsString,
    pub pixel_size: u32,
    pub byte_limit: usize,
}

/// Owned shell pixels tagged with the originating request for stale-result checks.
#[derive(Debug)]
pub struct ShellIconResponse {
    pub operation: OperationId,
    pub item: Guid,
    pub dib: ShellIconDib,
}

enum WorkerMessage {
    Load {
        request: ShellIconRequest,
        cancelled: Arc<AtomicBool>,
        response: SyncSender<Result<ShellIconResponse, PortError>>,
    },
    Shutdown,
}

/// One dedicated serial worker with a finite request queue.
///
/// A full queue fails immediately with `LimitExceeded`. Only pre-cancellation is
/// observable through this API: `Cancellation` contains a non-`Sync` `Cell`, and
/// `load` blocks its caller thread. Win32/COM calls cannot be interrupted safely; an
/// in-flight call completes and releases its native owners before its result drops.
/// Dropping this worker waits for that completion, as required for COM lifetime safety.
/// The owner must never drop it on a path that precedes desktop restoration.
pub struct ShellIconWorker {
    sender: Option<SyncSender<WorkerMessage>>,
    thread: Option<JoinHandle<()>>,
}

impl ShellIconWorker {
    /// Starts the worker with a caller-selected queue bound from 1 through 32.
    pub fn new(queue_limit: usize) -> Result<Self, PortError> {
        Self::with_probe(queue_limit, probe_shell_icon)
    }

    fn with_probe<F>(queue_limit: usize, probe: F) -> Result<Self, PortError>
    where
        F: FnMut(&OsStr, u32, usize) -> Result<ShellIconDib, PortError> + Send + 'static,
    {
        if !(1..=MAX_QUEUED_ICON_REQUESTS).contains(&queue_limit) {
            return Err(PortError::LimitExceeded);
        }
        let (sender, receiver) = mpsc::sync_channel(queue_limit);
        let thread = thread::Builder::new()
            .name("drape-shell-icons".into())
            .spawn(move || run_worker(receiver, probe))
            .map_err(|_| PortError::Unavailable)?;
        Ok(Self {
            sender: Some(sender),
            thread: Some(thread),
        })
    }

    /// Loads a raw, memory-only shell icon and returns owned bytes.
    ///
    /// The caller should run this blocking method on its bounded request worker,
    /// not the Slint event loop. `operation` and `item` are copied through so the
    /// UI bridge can reject stale results before doing event-loop image conversion.
    /// Cancellation during this blocking call cannot be observed through this API.
    pub fn load(
        &self,
        request: ShellIconRequest,
        cancellation: &Cancellation,
    ) -> Result<ShellIconResponse, PortError> {
        cancellation.check()?;
        if request.pixel_size == 0 || request.byte_limit == 0 {
            return Err(PortError::LimitExceeded);
        }

        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let cancelled = Arc::new(AtomicBool::new(false));
        let message = WorkerMessage::Load {
            request,
            cancelled,
            response: response_sender,
        };
        let sender = self.sender.as_ref().ok_or(PortError::Unavailable)?;
        match sender.try_send(message) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => return Err(PortError::LimitExceeded),
            Err(TrySendError::Disconnected(_)) => return Err(PortError::Unavailable),
        }

        let result = response_receiver
            .recv()
            .map_err(|_| PortError::Unavailable)?;
        cancellation.check()?;
        result
    }
}

impl Drop for ShellIconWorker {
    fn drop(&mut self) {
        // The owner must never reach this Drop before desktop restoration.
        if let Some(sender) = self.sender.take() {
            // Blocking send ensures a full bounded queue drains before shutdown.
            // The worker owns all in-flight native objects until the probe returns.
            let _ = sender.send(WorkerMessage::Shutdown);
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn run_worker<F>(receiver: Receiver<WorkerMessage>, mut probe: F)
where
    F: FnMut(&OsStr, u32, usize) -> Result<ShellIconDib, PortError>,
{
    while let Ok(message) = receiver.recv() {
        match message {
            WorkerMessage::Shutdown => break,
            WorkerMessage::Load {
                request,
                cancelled,
                response,
            } => {
                if cancelled.load(Ordering::Acquire) {
                    let _ = response.send(Err(PortError::Cancelled));
                    continue;
                }
                let result =
                    probe(&request.path, request.pixel_size, request.byte_limit).map(|dib| {
                        ShellIconResponse {
                            operation: request.operation,
                            item: request.item,
                            dib,
                        }
                    });
                // Discard abandoned pixels only after the probe releases native owners.
                let result = if cancelled.load(Ordering::Acquire) {
                    Err(PortError::Cancelled)
                } else {
                    result
                };
                let _ = response.send(result);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use std::sync::mpsc::{Receiver, Sender};

    type TestResponse = Receiver<Result<ShellIconResponse, PortError>>;
    type TestProbe = dyn FnMut(&OsStr, u32, usize) -> Result<ShellIconDib, PortError> + Send;

    fn request() -> ShellIconRequest {
        ShellIconRequest {
            operation: OperationId(3),
            item: Guid::parse("6f1c2b8e-0000-4000-8000-000000000100").unwrap(),
            path: OsString::from("unavailable-by-design"),
            pixel_size: 32,
            byte_limit: 4096,
        }
    }

    fn dib() -> ShellIconDib {
        ShellIconDib {
            width: 1,
            height: 1,
            stride: 4,
            bottom_up_bgrx: vec![0; 4],
        }
    }

    fn enqueue(worker: &ShellIconWorker, cancelled: Arc<AtomicBool>) -> TestResponse {
        let (response, receiver) = mpsc::sync_channel(1);
        let sent = worker
            .sender
            .as_ref()
            .unwrap()
            .try_send(WorkerMessage::Load {
                request: request(),
                cancelled,
                response,
            });
        assert!(sent.is_ok());
        receiver
    }

    fn blocking_probe(started: Sender<()>, release: Receiver<()>) -> Box<TestProbe> {
        let mut first_call = true;
        Box::new(move |_, _, _| {
            if first_call {
                first_call = false;
                let _ = started.send(());
                release.recv().unwrap();
            }
            Ok(dib())
        })
    }

    #[test]
    fn worker_rejects_unbounded_or_empty_queue_limits() {
        assert!(matches!(
            ShellIconWorker::new(0),
            Err(PortError::LimitExceeded)
        ));
        assert!(matches!(
            ShellIconWorker::new(MAX_QUEUED_ICON_REQUESTS + 1),
            Err(PortError::LimitExceeded)
        ));
    }

    #[test]
    fn pre_cancelled_request_never_reaches_the_native_worker() {
        let worker =
            ShellIconWorker::with_probe(1, |_, _, _| panic!("probe must not run")).unwrap();
        let cancellation = Cancellation::default();
        cancellation.cancel();

        assert!(matches!(
            worker.load(request(), &cancellation),
            Err(PortError::Cancelled)
        ));
    }

    #[test]
    fn full_queue_returns_limit_exceeded_without_blocking() {
        let (started_sender, started_receiver) = mpsc::channel();
        let (release_sender, release_receiver) = mpsc::channel();
        let worker =
            ShellIconWorker::with_probe(1, blocking_probe(started_sender, release_receiver))
                .unwrap();

        thread::scope(|scope| {
            let load_worker = &worker;
            let loading =
                scope.spawn(move || load_worker.load(request(), &Cancellation::default()));
            started_receiver.recv().unwrap();
            let _queued = enqueue(&worker, Arc::new(AtomicBool::new(false)));
            assert!(matches!(
                worker.load(request(), &Cancellation::default()),
                Err(PortError::LimitExceeded)
            ));
            release_sender.send(()).unwrap();
            assert!(loading.join().unwrap().is_ok());
        });
    }

    #[test]
    fn abandoned_queued_request_does_not_run_probe() {
        let (started_sender, started_receiver) = mpsc::channel();
        let (release_sender, release_receiver) = mpsc::channel();
        let calls = Arc::new(AtomicUsize::new(0));
        let probe_calls = Arc::clone(&calls);
        let worker = ShellIconWorker::with_probe(1, move |_, _, _| {
            if probe_calls.fetch_add(1, Ordering::SeqCst) == 0 {
                let _ = started_sender.send(());
                release_receiver.recv().unwrap();
            }
            Ok(dib())
        })
        .unwrap();

        thread::scope(|scope| {
            let load_worker = &worker;
            let loading =
                scope.spawn(move || load_worker.load(request(), &Cancellation::default()));
            started_receiver.recv().unwrap();
            let abandoned = Arc::new(AtomicBool::new(false));
            let queued_response = enqueue(&worker, Arc::clone(&abandoned));
            abandoned.store(true, Ordering::Release);
            release_sender.send(()).unwrap();
            assert!(loading.join().unwrap().is_ok());
            assert!(matches!(
                queued_response.recv().unwrap(),
                Err(PortError::Cancelled)
            ));
        });
        drop(worker);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn drop_waits_for_in_flight_probe() {
        let (started_sender, started_receiver) = mpsc::channel();
        let (release_sender, release_receiver) = mpsc::channel();
        let worker =
            ShellIconWorker::with_probe(1, blocking_probe(started_sender, release_receiver))
                .unwrap();
        let _response = enqueue(&worker, Arc::new(AtomicBool::new(false)));
        started_receiver.recv().unwrap();
        let (dropped_sender, dropped_receiver) = mpsc::channel();

        let dropping = thread::spawn(move || {
            drop(worker);
            dropped_sender.send(()).unwrap();
        });
        // A bounded wait gives a non-joining Drop time to report and fail the test.
        assert_eq!(
            dropped_receiver.recv_timeout(std::time::Duration::from_millis(200)),
            Err(mpsc::RecvTimeoutError::Timeout)
        );
        release_sender.send(()).unwrap();
        dropped_receiver.recv().unwrap();
        dropping.join().unwrap();
    }

    #[test]
    fn panicking_probe_makes_later_calls_unavailable() {
        let worker = ShellIconWorker::with_probe(1, |_, _, _| -> Result<ShellIconDib, PortError> {
            panic!("injected probe panic")
        })
        .unwrap();

        assert!(matches!(
            worker.load(request(), &Cancellation::default()),
            Err(PortError::Unavailable)
        ));
        assert!(matches!(
            worker.load(request(), &Cancellation::default()),
            Err(PortError::Unavailable)
        ));
    }
}
