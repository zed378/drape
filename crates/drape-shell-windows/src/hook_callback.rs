//! Bounded callback boundary for the future WH_MOUSE_LL owner.
//!
//! This module deliberately does not install a hook or call Windows APIs. It
//! keeps native callback policy separate from the serialized gesture consumer:
//! only fixed-size button fields are copied, admission never waits, and the
//! forwarding contract is exercised even when admission faults.
#![forbid(unsafe_code)]

use crate::hook_state::{Admission, ButtonEvent, ButtonSample, HookQueue};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, Ordering};

pub const WM_LBUTTONDOWN: u32 = 0x0201;
pub const WM_LBUTTONUP: u32 = 0x0202;
pub const WM_RBUTTONDOWN: u32 = 0x0204;
pub const WM_RBUTTONUP: u32 = 0x0205;
pub const WM_MBUTTONDOWN: u32 = 0x0207;
pub const WM_MBUTTONUP: u32 = 0x0208;
pub const WM_MOUSEWHEEL: u32 = 0x020A;
pub const WM_XBUTTONDOWN: u32 = 0x020B;
pub const WM_XBUTTONUP: u32 = 0x020C;
pub const WM_MOUSEHWHEEL: u32 = 0x020E;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeButtonMessage {
    pub message: u32,
    pub x: i32,
    pub y: i32,
    pub time_ms: u32,
}

/// State retained by the native callback. The queue itself owns all samples;
/// this state only gates admission and exposes aggregate fault information.
pub struct HookCallback {
    queue: HookQueue,
    accepting: AtomicBool,
    faulted: AtomicBool,
}

impl HookCallback {
    pub fn new(queue: HookQueue) -> Self {
        Self {
            queue,
            accepting: AtomicBool::new(true),
            faulted: AtomicBool::new(false),
        }
    }

    pub fn queue(&self) -> &HookQueue {
        &self.queue
    }

    pub fn disable(&self) {
        self.accepting.store(false, Ordering::Release);
        self.queue.mark_discontinuity();
    }

    pub fn enable(&self) {
        self.queue.mark_discontinuity();
        self.accepting.store(true, Ordering::Release);
    }

    pub fn faulted(&self) -> bool {
        self.faulted.load(Ordering::Acquire)
    }

    /// Handle the callback portion that is independent of the actual Windows
    /// ABI. Negative hook codes are forwarded without inspecting the message.
    /// The forwarding closure stands in for `CallNextHookEx` and is invoked on
    /// every normal return path.
    pub fn handle<F>(&self, code: i32, message: Option<NativeButtonMessage>, forward: F) -> isize
    where
        F: FnOnce() -> isize,
    {
        self.handle_with(code, || message, forward)
    }

    /// Handle a callback while deferring message decoding until after the hook
    /// code has been checked. A native ABI adapter can place its bounded
    /// `MSLLHOOKSTRUCT` read inside `read_message`; negative codes never invoke
    /// that closure. The callback path itself still performs no native read.
    pub fn handle_with<ReadMessage, Forward>(
        &self,
        code: i32,
        read_message: ReadMessage,
        forward: Forward,
    ) -> isize
    where
        ReadMessage: FnOnce() -> Option<NativeButtonMessage>,
        Forward: FnOnce() -> isize,
    {
        if code >= 0 {
            let admission = catch_unwind(AssertUnwindSafe(|| self.admit(read_message())));
            if admission.is_err() {
                self.faulted.store(true, Ordering::Release);
                self.queue.mark_discontinuity();
            }
        }
        catch_unwind(AssertUnwindSafe(forward)).unwrap_or_default()
    }

    fn admit(&self, message: Option<NativeButtonMessage>) -> Admission {
        if !self.accepting.load(Ordering::Acquire) {
            self.queue.mark_discontinuity();
            return Admission::Dropped;
        }
        let Some(message) = message else {
            return Admission::Dropped;
        };
        if matches!(
            message.message,
            WM_MBUTTONDOWN
                | WM_MBUTTONUP
                | WM_MOUSEWHEEL
                | WM_XBUTTONDOWN
                | WM_XBUTTONUP
                | WM_MOUSEHWHEEL
        ) {
            self.queue.mark_discontinuity();
            return Admission::Dropped;
        }
        let event = match message.message {
            WM_LBUTTONDOWN => ButtonEvent::LeftDown,
            WM_LBUTTONUP => ButtonEvent::LeftUp,
            WM_RBUTTONDOWN => ButtonEvent::RightDown,
            WM_RBUTTONUP => ButtonEvent::RightUp,
            _ => return Admission::Dropped,
        };
        self.queue.try_push(ButtonSample {
            event,
            x: message.x,
            y: message.y,
            time_ms: message.time_ms,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hook_state::QueuePoll;
    use std::num::NonZeroUsize;

    fn callback() -> HookCallback {
        HookCallback::new(HookQueue::new(NonZeroUsize::new(2).unwrap()).unwrap())
    }

    fn msg(message: u32) -> NativeButtonMessage {
        NativeButtonMessage {
            message,
            x: -4,
            y: 8,
            time_ms: 9,
        }
    }

    #[test]
    fn negative_code_forwards_without_admitting_input() {
        let callback = callback();
        assert_eq!(
            callback.handle_with(
                -1,
                || panic!("negative hook codes must not inspect callback data"),
                || 17,
            ),
            17
        );
        assert_eq!(callback.queue().poll(), QueuePoll::Empty);
    }

    #[test]
    fn message_read_panic_latches_a_reset_and_still_forwards() {
        let callback = callback();
        assert_eq!(
            callback.handle_with(0, || panic!("malformed callback adapter"), || 23),
            23
        );
        assert!(callback.faulted());
        assert_eq!(callback.queue().poll(), QueuePoll::Reset);
    }

    #[test]
    fn eligible_button_is_copied_and_forwarded() {
        let callback = callback();
        assert_eq!(callback.handle(0, Some(msg(WM_LBUTTONDOWN)), || 3), 3);
        assert!(matches!(
            callback.queue().poll(),
            QueuePoll::Event(ButtonSample {
                event: ButtonEvent::LeftDown,
                x: -4,
                ..
            })
        ));
    }

    #[test]
    fn unknown_message_is_forwarded_without_queue_entry() {
        let callback = callback();
        assert_eq!(callback.handle(0, Some(msg(0x1234)), || 4), 4);
        assert_eq!(callback.queue().poll(), QueuePoll::Empty);
    }

    #[test]
    fn other_button_and_wheel_messages_break_left_click_sequences() {
        for message in [
            WM_MBUTTONDOWN,
            WM_MBUTTONUP,
            WM_MOUSEWHEEL,
            WM_XBUTTONDOWN,
            WM_XBUTTONUP,
            WM_MOUSEHWHEEL,
        ] {
            let callback = callback();
            for button_message in [WM_LBUTTONDOWN, WM_LBUTTONUP] {
                assert_eq!(callback.handle(0, Some(msg(button_message)), || 5), 5);
                assert!(matches!(callback.queue().poll(), QueuePoll::Event(_)));
            }
            assert_eq!(callback.handle(0, Some(msg(message)), || 6), 6);
            assert_eq!(callback.queue().poll(), QueuePoll::Reset);
            for button_message in [WM_LBUTTONDOWN, WM_LBUTTONUP] {
                assert_eq!(callback.handle(0, Some(msg(button_message)), || 7), 7);
                assert!(matches!(callback.queue().poll(), QueuePoll::Event(_)));
            }
        }
    }

    #[test]
    fn disable_latches_loss_and_drops_until_enabled() {
        let callback = callback();
        callback.disable();
        callback.handle(0, Some(msg(WM_LBUTTONDOWN)), || 1);
        assert_eq!(callback.queue().poll(), QueuePoll::Reset);
        callback.enable();
        callback.handle(0, Some(msg(WM_LBUTTONDOWN)), || 1);
        assert_eq!(callback.queue().poll(), QueuePoll::Reset);
        callback.handle(0, Some(msg(WM_LBUTTONUP)), || 1);
        assert!(matches!(
            callback.queue().poll(),
            QueuePoll::Event(ButtonSample {
                event: ButtonEvent::LeftUp,
                ..
            })
        ));
    }

    #[test]
    fn forwarding_panic_is_contained() {
        let callback = callback();
        assert_eq!(callback.handle(-1, None, || panic!("forward")), 0);
        assert!(!callback.faulted());
    }
}
