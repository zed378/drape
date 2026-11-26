//! Bounded, single-threaded STA dispatch for the synchronous shell icon probe.
//!
//! This returns raw DIB bytes. Channel order and alpha semantics remain unverified,
//! so callers must not reinterpret the response as a Core `IconBitmap` or UI image.

use crate::shell_items::{ShellIconDib, probe_shell_icon};
use drape_core::domain::Guid;
use drape_core::ports::{Cancellation, OperationId, PortError};
use std::ffi::OsString;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, SyncSender, TrySendError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

const MAX_QUEUED_ICON_REQUESTS: usize = 32;
const CANCELLATION_POLL: Duration = Duration::from_millis(10);

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
/// A full queue fails immediately with `LimitExceeded`. Cancellation while waiting
/// abandons the caller's response and asks the worker to skip work if it has not
/// started. Win32/COM calls are synchronous and cannot be interrupted safely; an
/// in-flight call completes and releases its native owners before its result drops.
/// Dropping this worker waits for that completion, as required for COM lifetime safety.
pub struct ShellIconWorker {
    sender: Option<SyncSender<WorkerMessage>>,
    thread: Option<JoinHandle<()>>,
}

impl ShellIconWorker {
    /// Starts the worker with a caller-selected queue bound from 1 through 32.
    pub fn new(queue_limit: usize) -> Result<Self, PortError> {
        if !(1..=MAX_QUEUED_ICON_REQUESTS).contains(&queue_limit) {
            return Err(PortError::LimitExceeded);
        }
        let (sender, receiver) = mpsc::sync_channel(queue_limit);
        let thread = thread::Builder::new()
            .name("drape-shell-icons".into())
            .spawn(move || run_worker(receiver))
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
            cancelled: Arc::clone(&cancelled),
            response: response_sender,
        };
        let sender = self.sender.as_ref().ok_or(PortError::Unavailable)?;
        match sender.try_send(message) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => return Err(PortError::LimitExceeded),
            Err(TrySendError::Disconnected(_)) => return Err(PortError::Unavailable),
        }

        loop {
            if cancellation.is_cancelled() {
                cancelled.store(true, Ordering::Release);
                return Err(PortError::Cancelled);
            }
            match response_receiver.recv_timeout(CANCELLATION_POLL) {
                Ok(result) => {
                    cancellation.check()?;
                    return result;
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return Err(PortError::Unavailable),
            }
        }
    }
}

impl Drop for ShellIconWorker {
    fn drop(&mut self) {
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

fn run_worker(receiver: Receiver<WorkerMessage>) {
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
                    probe_shell_icon(&request.path, request.pixel_size, request.byte_limit).map(
                        |dib| ShellIconResponse {
                            operation: request.operation,
                            item: request.item,
                            dib,
                        },
                    );
                // Cancellation cannot abort the synchronous native call. Discard
                // any completed pixels after probe_shell_icon has released owners.
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
        let worker = ShellIconWorker::new(1).unwrap();
        let cancellation = Cancellation::default();
        cancellation.cancel();
        let request = ShellIconRequest {
            operation: OperationId(3),
            item: Guid::parse("6f1c2b8e-0000-4000-8000-000000000100").unwrap(),
            path: OsString::from("unavailable-by-design"),
            pixel_size: 32,
            byte_limit: 4096,
        };

        assert!(matches!(
            worker.load(request, &cancellation),
            Err(PortError::Cancelled)
        ));
    }
}
