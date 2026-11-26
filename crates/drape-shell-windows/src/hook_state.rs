//! Pure R0-08 transient input state. This module installs no native hook.
#![forbid(unsafe_code)]

use drape_core::ports::PortError;
use std::fmt;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, TryLockError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonEvent {
    LeftDown,
    LeftUp,
    RightDown,
    RightUp,
}

/// Fixed-size transient input. Debug intentionally never reveals input values.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ButtonSample {
    pub event: ButtonEvent,
    pub x: i32,
    pub y: i32,
    pub time_ms: u32,
}

impl fmt::Debug for ButtonSample {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ButtonSample(<redacted>)")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Admission {
    Enqueued,
    Dropped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueuePoll {
    Empty,
    Event(ButtonSample),
    Reset,
}

/// Bounded, non-blocking admission; not a lock-free or wait-free queue.
/// One intended producer and consumer; no native callback storage is installed.
pub struct HookQueue {
    ring: Mutex<Ring>,
    lost: AtomicBool,
}

struct Ring {
    slots: Vec<Option<ButtonSample>>,
    head: usize,
    len: usize,
}

impl Ring {
    fn clear(&mut self) {
        self.slots.fill(None);
        self.head = 0;
        self.len = 0;
    }
}

impl HookQueue {
    /// Allocate once, before callback admission. Oversized/allocation failures
    /// are redacted; successful construction never grows this storage.
    pub fn new(capacity: NonZeroUsize) -> Result<Self, PortError> {
        let mut slots = Vec::new();
        slots
            .try_reserve_exact(capacity.get())
            .map_err(|_| PortError::LimitExceeded)?;
        slots.resize(capacity.get(), None);
        Ok(Self {
            ring: Mutex::new(Ring {
                slots,
                head: 0,
                len: 0,
            }),
            lost: AtomicBool::new(false),
        })
    }

    /// Never waits for the consumer or allocates. A loss closes admission until
    /// the consumer observes Reset; an input sequence cannot bridge that gap.
    pub fn try_push(&self, sample: ButtonSample) -> Admission {
        if self.lost.load(Ordering::Acquire) {
            return Admission::Dropped;
        }
        let mut ring = match self.ring.try_lock() {
            Ok(ring) => ring,
            Err(_) => {
                self.mark_discontinuity();
                return Admission::Dropped;
            }
        };
        // Recheck under the ring lock: a previous admission attempt or explicit
        // cancellation may have latched loss since our first observation.
        if self.lost.load(Ordering::Acquire) {
            return Admission::Dropped;
        }
        if ring.len == ring.slots.len() {
            self.mark_discontinuity();
            return Admission::Dropped;
        }
        // Slots are allocated objects; their total size is bounded by isize::MAX.
        // Thus two valid slot indexes cannot overflow usize here.
        let tail = (ring.head + ring.len) % ring.slots.len();
        ring.slots[tail] = Some(sample);
        ring.len += 1;
        Admission::Enqueued
    }

    /// Poll on the single gesture consumer. The loss check orders this poll: a
    /// concurrent discontinuity after it may follow one already-claimed event.
    /// Handle that event synchronously, process Reset before later events, and
    /// revalidate enabled state and host generation before dispatching intent.
    /// A poisoned ring remains disabled until replaced.
    pub fn poll(&self) -> QueuePoll {
        let mut ring = match self.ring.try_lock() {
            Ok(ring) => ring,
            Err(TryLockError::WouldBlock) => return QueuePoll::Empty,
            Err(TryLockError::Poisoned(error)) => {
                // The guard is still exclusively owned even after poisoning.
                // Discard retained input, but do not clear poison/re-enable it.
                error.into_inner().clear();
                self.mark_discontinuity();
                return QueuePoll::Reset;
            }
        };
        if self.lost.swap(false, Ordering::AcqRel) {
            ring.clear();
            return QueuePoll::Reset;
        }
        if ring.len == 0 {
            return QueuePoll::Empty;
        }
        let head = ring.head;
        let sample = ring.slots[head].take();
        ring.head = (head + 1) % ring.slots.len();
        ring.len -= 1;
        match sample {
            Some(sample) => QueuePoll::Event(sample),
            None => {
                // Fail closed if an internal invariant is ever violated.
                ring.clear();
                QueuePoll::Reset
            }
        }
    }

    /// Latch host/session loss, disable, shutdown or cancellation. This does not
    /// stop native execution; the owning lifecycle must unhook/join separately.
    pub fn mark_discontinuity(&self) {
        self.lost.store(true, Ordering::Release);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoubleClickSettings {
    time_ms: u32,
    width_px: u32,
    height_px: u32,
}

impl DoubleClickSettings {
    pub fn new(time_ms: u32, width_px: u32, height_px: u32) -> Result<Self, PortError> {
        if time_ms == 0 || time_ms >= (1 << 31) || width_px == 0 || height_px == 0 {
            return Err(PortError::Unavailable);
        }
        Ok(Self {
            time_ms,
            width_px,
            height_px,
        })
    }
}

/// Supplied only by a verified off-callback hit test. Unknown is never empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopHit {
    Empty { generation: u64 },
    Occupied,
    Unknown,
}

/// Bounded transient state; the caller resets on queue loss, idle timeout,
/// disable, drag/movement cancellation and host/session replacement.
/// Button-only samples cannot prove absence of intervening pointer movement.
pub struct DoubleClickRecognizer {
    settings: DoubleClickSettings,
    pressed: Option<ClickStart>,
    candidate: Option<ClickStart>,
}

#[derive(Clone, Copy)]
struct ClickStart {
    sample: ButtonSample,
    generation: u64,
}

impl DoubleClickRecognizer {
    pub fn new(settings: DoubleClickSettings) -> Self {
        Self {
            settings,
            pressed: None,
            candidate: None,
        }
    }

    /// Supplied hits must describe this sample on the current validated desktop.
    /// A true result is gesture intent only, never permission to hide icons.
    pub fn observe(&mut self, sample: ButtonSample, hit: DesktopHit) -> bool {
        let DesktopHit::Empty { generation } = hit else {
            self.reset();
            return false;
        };
        match sample.event {
            ButtonEvent::LeftDown => {
                if self.pressed.is_some() {
                    self.reset();
                    return false;
                }
                if self
                    .candidate
                    .is_some_and(|start| !self.compatible(start, sample, generation))
                {
                    self.candidate = None;
                }
                self.pressed = Some(ClickStart { sample, generation });
                false
            }
            ButtonEvent::LeftUp => {
                let Some(pressed) = self.pressed.take() else {
                    self.reset();
                    return false;
                };
                if !self.compatible(pressed, sample, generation) {
                    self.reset();
                    return false;
                }
                // The system interval compares the two downs. The second up may
                // occur after that deadline, provided its own press is bounded.
                if self
                    .candidate
                    .take()
                    .is_some_and(|first| self.compatible(first, pressed.sample, pressed.generation))
                {
                    self.reset();
                    true
                } else {
                    self.candidate = Some(pressed);
                    false
                }
            }
            ButtonEvent::RightDown | ButtonEvent::RightUp => {
                self.reset();
                false
            }
        }
    }

    fn compatible(&self, start: ClickStart, sample: ButtonSample, generation: u64) -> bool {
        start.generation == generation
            && sample.time_ms.wrapping_sub(start.sample.time_ms) <= self.settings.time_ms
            && within_extent(start.sample.x, sample.x, self.settings.width_px)
            && within_extent(start.sample.y, sample.y, self.settings.height_px)
    }

    pub fn reset(&mut self) {
        self.pressed = None;
        self.candidate = None;
    }
}

/// Consumer-visible context, supplied from the serialized owner that controls
/// hook enablement and validates the current desktop/session generation.
/// Generation must change whenever either host or session identity changes;
/// reusing a generation cannot detect a context that changes away and back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConsumerContext {
    pub enabled: bool,
    pub generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsumerOutcome {
    /// No queued sample was available; the current context was observed.
    Empty,
    /// A queue discontinuity was consumed and recognizer history was cleared.
    Reset,
    /// A sample was discarded while the feature was disabled.
    Disabled,
    /// Context changed; queued samples are latched for reset before later input.
    ContextChanged,
    /// One sample was consumed without completing a gesture.
    SampleConsumed,
    /// Gesture intent was synchronously handed to the supplied dispatcher.
    IntentDispatched,
}

/// Pure serialized consumer state. The native owner must run polling, context
/// updates and intent dispatch on one consumer thread; this type installs no
/// hook and does not implement a desktop hit test.
pub struct HookConsumer {
    recognizer: DoubleClickRecognizer,
    last_context: Option<ConsumerContext>,
}

impl HookConsumer {
    /// Create an idle consumer with no validated context yet.
    pub fn new(settings: DoubleClickSettings) -> Self {
        Self {
            recognizer: DoubleClickRecognizer::new(settings),
            last_context: None,
        }
    }

    /// Cancel immediately on the serialized owner (including idle timeout,
    /// movement/drag, disable or session loss), then discard queued history on
    /// the next poll. The owner must supply the timeout signal even when idle;
    /// this pure consumer has no clock or timer. Call this for disable/re-enable
    /// transitions even when no poll occurs between them.
    pub fn reset(&mut self, queue: &HookQueue) {
        self.recognizer.reset();
        self.last_context = None;
        queue.mark_discontinuity();
    }

    /// Poll and handle at most one bounded sample. Hit testing runs on this
    /// serialized consumer, never in the native callback. The caller supplies a
    /// bounded hit test for a validated desktop target and a synchronous intent
    /// dispatcher. Context is checked before and after hit testing, then again
    /// immediately before dispatch. A context change latches queue loss so Reset
    /// is observed before later samples. Initial context validation (also after
    /// reset) emits Reset before samples; a contended poll may first yield Empty.
    pub fn poll_one<Context, HitTest, Dispatch>(
        &mut self,
        queue: &HookQueue,
        mut context: Context,
        mut hit_test: HitTest,
        mut dispatch: Dispatch,
    ) -> ConsumerOutcome
    where
        Context: FnMut() -> ConsumerContext,
        HitTest: FnMut(i32, i32) -> DesktopHit,
        Dispatch: FnMut(u64),
    {
        if self.last_context.is_none() {
            queue.mark_discontinuity();
        }
        match queue.poll() {
            QueuePoll::Empty => {
                let current = context();
                if self.last_context.is_some_and(|last| last != current) {
                    self.recognizer.reset();
                    self.last_context = Some(current);
                    queue.mark_discontinuity();
                    ConsumerOutcome::ContextChanged
                } else {
                    self.last_context = Some(current);
                    ConsumerOutcome::Empty
                }
            }
            QueuePoll::Reset => {
                self.recognizer.reset();
                self.last_context = Some(context());
                ConsumerOutcome::Reset
            }
            QueuePoll::Event(sample) => {
                let before = context();
                if self.last_context.is_some_and(|last| last != before) {
                    self.recognizer.reset();
                    self.last_context = Some(before);
                    queue.mark_discontinuity();
                    return ConsumerOutcome::ContextChanged;
                }
                self.last_context = Some(before);
                if !before.enabled {
                    self.recognizer.reset();
                    queue.mark_discontinuity();
                    return ConsumerOutcome::Disabled;
                }

                let hit = hit_test(sample.x, sample.y);
                let after_hit_test = context();
                if after_hit_test != before {
                    self.recognizer.reset();
                    self.last_context = Some(after_hit_test);
                    queue.mark_discontinuity();
                    return ConsumerOutcome::ContextChanged;
                }

                let hit = match hit {
                    DesktopHit::Empty { generation } if generation == before.generation => {
                        DesktopHit::Empty { generation }
                    }
                    DesktopHit::Empty { .. } | DesktopHit::Occupied | DesktopHit::Unknown => {
                        DesktopHit::Unknown
                    }
                };
                if !self.recognizer.observe(sample, hit) {
                    return ConsumerOutcome::SampleConsumed;
                }

                let before_dispatch = context();
                if before_dispatch != before {
                    self.recognizer.reset();
                    self.last_context = Some(before_dispatch);
                    queue.mark_discontinuity();
                    return ConsumerOutcome::ContextChanged;
                }

                dispatch(before.generation);
                ConsumerOutcome::IntentDispatched
            }
        }
    }
}

fn within_extent(first: i32, second: i32, extent: u32) -> bool {
    let delta = i64::from(second) - i64::from(first);
    let negative = i64::from(extent / 2);
    delta >= -negative && delta < i64::from(extent) - negative
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(event: ButtonEvent, x: i32, y: i32, time_ms: u32) -> ButtonSample {
        ButtonSample {
            event,
            x,
            y,
            time_ms,
        }
    }

    fn empty(generation: u64) -> DesktopHit {
        DesktopHit::Empty { generation }
    }

    fn recognizer() -> DoubleClickRecognizer {
        DoubleClickRecognizer::new(DoubleClickSettings::new(100, 10, 10).unwrap())
    }

    fn click(
        recognizer: &mut DoubleClickRecognizer,
        x: i32,
        y: i32,
        down: u32,
        up: u32,
        generation: u64,
    ) -> bool {
        assert!(!recognizer.observe(sample(ButtonEvent::LeftDown, x, y, down), empty(generation)));
        recognizer.observe(sample(ButtonEvent::LeftUp, x, y, up), empty(generation))
    }

    #[test]
    fn queue_fifo_overflow_resets_all_history_before_new_admission() {
        let queue = HookQueue::new(NonZeroUsize::new(2).unwrap()).unwrap();
        let one = sample(ButtonEvent::LeftDown, -20, 30, 1);
        let two = sample(ButtonEvent::LeftUp, -20, 30, 2);
        assert_eq!(queue.try_push(one), Admission::Enqueued);
        assert_eq!(queue.try_push(two), Admission::Enqueued);
        assert_eq!(queue.poll(), QueuePoll::Event(one));
        assert_eq!(queue.poll(), QueuePoll::Event(two));
        assert_eq!(queue.poll(), QueuePoll::Empty);
        assert_eq!(queue.try_push(one), Admission::Enqueued);
        assert_eq!(queue.try_push(two), Admission::Enqueued);
        assert_eq!(queue.try_push(one), Admission::Dropped);
        assert_eq!(queue.try_push(two), Admission::Dropped);
        assert_eq!(queue.poll(), QueuePoll::Reset);
        assert_eq!(queue.poll(), QueuePoll::Empty);
        assert_eq!(queue.try_push(two), Admission::Enqueued);
        assert_eq!(queue.poll(), QueuePoll::Event(two));
    }

    #[test]
    fn queue_explicit_discontinuity_discards_pending_samples() {
        let queue = HookQueue::new(NonZeroUsize::new(1).unwrap()).unwrap();
        assert_eq!(
            queue.try_push(sample(ButtonEvent::LeftDown, 1, 2, 3)),
            Admission::Enqueued
        );
        queue.mark_discontinuity();
        queue.mark_discontinuity();
        assert_eq!(queue.poll(), QueuePoll::Reset);
        assert_eq!(queue.poll(), QueuePoll::Empty);
    }

    #[test]
    fn queue_constructor_rejects_impossible_capacity_without_panicking() {
        assert!(HookQueue::new(NonZeroUsize::new(usize::MAX).unwrap()).is_err());
    }

    #[test]
    fn queue_contention_marks_loss_without_waiting_or_replaying_old_events() {
        let queue = HookQueue::new(NonZeroUsize::new(2).unwrap()).unwrap();
        let event = sample(ButtonEvent::LeftDown, 1, 2, 3);
        assert_eq!(queue.try_push(event), Admission::Enqueued);
        // Deterministic same-thread contention: blocking admission would deadlock.
        // No sleeps, scheduler assumptions or callback/native installation.
        let guard = queue.ring.lock().unwrap();
        assert_eq!(queue.try_push(event), Admission::Dropped);
        assert_eq!(queue.poll(), QueuePoll::Empty);
        drop(guard);
        assert_eq!(queue.try_push(event), Admission::Dropped);
        assert_eq!(queue.poll(), QueuePoll::Reset);
        assert_eq!(queue.poll(), QueuePoll::Empty);
        assert_eq!(queue.try_push(event), Admission::Enqueued);
        assert_eq!(queue.poll(), QueuePoll::Event(event));
    }

    #[test]
    fn queue_poison_fails_closed_until_replaced() {
        let queue = HookQueue::new(NonZeroUsize::new(1).unwrap()).unwrap();
        assert_eq!(
            queue.try_push(sample(ButtonEvent::LeftDown, 1, 2, 3)),
            Admission::Enqueued
        );
        let result = std::panic::catch_unwind(|| {
            let _guard = queue.ring.lock().unwrap();
            panic!("injected queue owner failure");
        });
        assert!(result.is_err());
        assert_eq!(
            queue.try_push(sample(ButtonEvent::LeftDown, 1, 2, 3)),
            Admission::Dropped
        );
        assert_eq!(queue.poll(), QueuePoll::Reset);
        let guard = match queue.ring.lock() {
            Ok(_) => panic!("poison must remain latched"),
            Err(error) => error.into_inner(),
        };
        assert_eq!(guard.len, 0);
        assert!(guard.slots.iter().all(Option::is_none));
        drop(guard);
        assert_eq!(queue.poll(), QueuePoll::Reset);
    }

    #[test]
    fn input_debug_is_redacted() {
        assert_eq!(
            format!(
                "{:?}",
                sample(ButtonEvent::LeftDown, 123456, -654321, 987654)
            ),
            "ButtonSample(<redacted>)"
        );
    }

    #[test]
    fn double_click_requires_complete_pairs_and_does_not_reuse_emitted_clicks() {
        let mut state = recognizer();
        assert!(!click(&mut state, -100, 25, 10, 20, 7));
        assert!(click(&mut state, -100, 25, 30, 40, 7));
        assert!(!click(&mut state, -100, 25, 50, 60, 7));
        assert!(click(&mut state, -100, 25, 70, 80, 7));
    }

    #[test]
    fn double_click_rejects_cross_generation_unknown_and_occupied_endpoints() {
        for failure in [empty(8), DesktopHit::Unknown, DesktopHit::Occupied] {
            let mut state = recognizer();
            assert!(!click(&mut state, 0, 0, 10, 20, 7));
            assert!(!state.observe(sample(ButtonEvent::LeftDown, 0, 0, 30), failure));
            assert!(!state.observe(sample(ButtonEvent::LeftUp, 0, 0, 40), empty(7)));
        }
        let mut state = recognizer();
        assert!(!click(&mut state, 0, 0, 10, 20, 7));
        assert!(!state.observe(sample(ButtonEvent::LeftDown, 0, 0, 30), empty(7)));
        assert!(!state.observe(sample(ButtonEvent::LeftUp, 0, 0, 40), empty(8)));
    }

    #[test]
    fn double_click_uses_down_interval_and_half_open_system_rectangle() {
        for (x, y, expected) in [
            (4, 4, true),
            (-5, -5, true),
            (5, 0, false),
            (0, 5, false),
            (-6, 0, false),
        ] {
            let mut state = recognizer();
            assert!(!click(&mut state, 0, 0, 100, 110, 1));
            assert_eq!(click(&mut state, x, y, 200, 210, 1), expected);
        }
        let mut state = recognizer();
        assert!(!click(&mut state, 0, 0, 100, 110, 1));
        assert!(!click(&mut state, 0, 0, 201, 210, 1));
    }

    #[test]
    fn double_click_handles_wrapping_time_and_extreme_coordinates_without_overflow() {
        let mut state = recognizer();
        assert!(!click(
            &mut state,
            i32::MIN,
            i32::MAX,
            u32::MAX - 20,
            u32::MAX - 10,
            1
        ));
        assert!(click(&mut state, i32::MIN + 1, i32::MAX - 1, 5, 10, 1));
        let mut state = recognizer();
        assert!(!click(&mut state, i32::MIN, i32::MIN, 10, 20, 1));
        assert!(!click(&mut state, i32::MAX, i32::MAX, 30, 40, 1));
        let mut state = recognizer();
        assert!(!click(&mut state, 0, 0, 100, 110, 1));
        assert!(!click(&mut state, 0, 0, 90, 95, 1));
    }

    #[test]
    fn double_click_reset_right_button_and_malformed_pairs_cancel_candidates() {
        let mut state = recognizer();
        assert!(!click(&mut state, 0, 0, 10, 20, 1));
        state.reset();
        assert!(!click(&mut state, 0, 0, 30, 40, 1));
        assert!(!state.observe(sample(ButtonEvent::RightDown, 0, 0, 45), empty(1)));
        assert!(!state.observe(sample(ButtonEvent::RightUp, 0, 0, 46), empty(1)));
        assert!(!click(&mut state, 0, 0, 50, 60, 1));
        assert!(!state.observe(sample(ButtonEvent::LeftUp, 0, 0, 65), empty(1)));
        assert!(!click(&mut state, 0, 0, 70, 80, 1));
        assert!(!state.observe(sample(ButtonEvent::LeftDown, 0, 0, 85), empty(1)));
        assert!(!state.observe(sample(ButtonEvent::LeftDown, 0, 0, 86), empty(1)));
        assert!(!state.observe(sample(ButtonEvent::LeftUp, 0, 0, 87), empty(1)));
    }

    #[test]
    fn double_click_rejects_invalid_settings_and_invalid_press_release() {
        for settings in [(0, 1, 1), (1, 0, 1), (1, 1, 0), (1 << 31, 1, 1)] {
            assert!(DoubleClickSettings::new(settings.0, settings.1, settings.2).is_err());
        }
        let mut state = recognizer();
        assert!(!click(&mut state, 0, 0, 10, 111, 1));
        assert!(!click(&mut state, 0, 0, 120, 130, 1));
        assert!(!state.observe(sample(ButtonEvent::LeftDown, 0, 0, 140), empty(1)));
        assert!(!state.observe(sample(ButtonEvent::LeftUp, 20, 20, 150), empty(1)));
    }

    fn consumer_context(enabled: bool, generation: u64) -> ConsumerContext {
        ConsumerContext {
            enabled,
            generation,
        }
    }

    fn consume_one(
        consumer: &mut HookConsumer,
        queue: &HookQueue,
        context: &std::cell::Cell<ConsumerContext>,
        hit_test: impl FnMut(i32, i32) -> DesktopHit,
        dispatched: &mut Vec<u64>,
    ) -> ConsumerOutcome {
        consumer.poll_one(
            queue,
            || context.get(),
            hit_test,
            |generation| dispatched.push(generation),
        )
    }

    fn click_samples(down: u32, up: u32) -> [ButtonSample; 2] {
        [
            sample(ButtonEvent::LeftDown, -5, 10, down),
            sample(ButtonEvent::LeftUp, -5, 10, up),
        ]
    }

    fn initialized_consumer(queue: &HookQueue, context: ConsumerContext) -> HookConsumer {
        let mut consumer = HookConsumer::new(DoubleClickSettings::new(100, 10, 10).unwrap());
        assert_eq!(
            consumer.poll_one(
                queue,
                || context,
                |_, _| panic!("initial validation must not hit test"),
                |_| panic!("initial validation must not dispatch"),
            ),
            ConsumerOutcome::Reset
        );
        consumer
    }

    #[test]
    fn consumer_first_poll_discards_samples_queued_before_context_validation() {
        let queue = HookQueue::new(NonZeroUsize::new(4).unwrap()).unwrap();
        for event in click_samples(10, 20)
            .into_iter()
            .chain(click_samples(30, 40))
        {
            assert_eq!(queue.try_push(event), Admission::Enqueued);
        }
        let mut consumer = HookConsumer::new(DoubleClickSettings::new(100, 10, 10).unwrap());
        assert_eq!(
            consumer.poll_one(
                &queue,
                || consumer_context(true, 7),
                |_, _| panic!("unvalidated queued input must not reach hit testing"),
                |_| panic!("unvalidated queued input must not dispatch"),
            ),
            ConsumerOutcome::Reset
        );
        assert_eq!(queue.poll(), QueuePoll::Empty);
        let mut dispatched = Vec::new();
        for event in click_samples(50, 60)
            .into_iter()
            .chain(click_samples(70, 80))
        {
            assert_eq!(queue.try_push(event), Admission::Enqueued);
            consumer.poll_one(
                &queue,
                || consumer_context(true, 7),
                |_, _| empty(7),
                |generation| dispatched.push(generation),
            );
        }
        assert_eq!(dispatched, [7]);
    }

    #[test]
    fn consumer_dispatches_once_after_two_complete_current_empty_desktop_clicks() {
        let queue = HookQueue::new(NonZeroUsize::new(4).unwrap()).unwrap();
        let context = std::cell::Cell::new(consumer_context(true, 7));
        let mut consumer = initialized_consumer(&queue, context.get());
        let mut dispatched = Vec::new();

        for (index, event) in click_samples(10, 20)
            .into_iter()
            .chain(click_samples(30, 40))
            .enumerate()
        {
            assert_eq!(queue.try_push(event), Admission::Enqueued);
            let outcome = consume_one(
                &mut consumer,
                &queue,
                &context,
                |_, _| empty(7),
                &mut dispatched,
            );
            let expected = if index == 3 {
                ConsumerOutcome::IntentDispatched
            } else {
                ConsumerOutcome::SampleConsumed
            };
            assert_eq!(outcome, expected);
        }

        assert_eq!(dispatched, [7]);
    }

    #[test]
    fn consumer_reset_discards_candidate_before_post_discontinuity_clicks() {
        let queue = HookQueue::new(NonZeroUsize::new(4).unwrap()).unwrap();
        let context = std::cell::Cell::new(consumer_context(true, 7));
        let mut consumer = initialized_consumer(&queue, context.get());
        let mut dispatched = Vec::new();

        for event in click_samples(10, 20) {
            assert_eq!(queue.try_push(event), Admission::Enqueued);
            assert_eq!(
                consume_one(
                    &mut consumer,
                    &queue,
                    &context,
                    |_, _| empty(7),
                    &mut dispatched,
                ),
                ConsumerOutcome::SampleConsumed
            );
        }
        queue.mark_discontinuity();
        assert_eq!(
            consume_one(
                &mut consumer,
                &queue,
                &context,
                |_, _| empty(7),
                &mut dispatched,
            ),
            ConsumerOutcome::Reset
        );

        for event in click_samples(30, 40) {
            assert_eq!(queue.try_push(event), Admission::Enqueued);
            assert_eq!(
                consume_one(
                    &mut consumer,
                    &queue,
                    &context,
                    |_, _| empty(7),
                    &mut dispatched,
                ),
                ConsumerOutcome::SampleConsumed
            );
        }
        assert!(dispatched.is_empty());
    }

    #[test]
    fn consumer_revalidates_generation_after_hit_test_and_resets_queued_samples() {
        let queue = HookQueue::new(NonZeroUsize::new(4).unwrap()).unwrap();
        let context = std::cell::Cell::new(consumer_context(true, 7));
        let mut consumer = initialized_consumer(&queue, context.get());
        let mut dispatched = Vec::new();

        for event in click_samples(10, 20) {
            assert_eq!(queue.try_push(event), Admission::Enqueued);
            assert_eq!(
                consume_one(
                    &mut consumer,
                    &queue,
                    &context,
                    |_, _| empty(7),
                    &mut dispatched,
                ),
                ConsumerOutcome::SampleConsumed
            );
        }
        let second = click_samples(30, 40);
        assert_eq!(queue.try_push(second[0]), Admission::Enqueued);
        assert_eq!(queue.try_push(second[1]), Admission::Enqueued);

        assert_eq!(
            consumer.poll_one(
                &queue,
                || context.get(),
                |_, _| {
                    context.set(consumer_context(true, 8));
                    empty(7)
                },
                |generation| dispatched.push(generation),
            ),
            ConsumerOutcome::ContextChanged
        );
        assert_eq!(
            consume_one(
                &mut consumer,
                &queue,
                &context,
                |_, _| empty(8),
                &mut dispatched,
            ),
            ConsumerOutcome::Reset
        );
        assert!(dispatched.is_empty());
    }

    #[test]
    fn consumer_revalidates_enabled_and_generation_immediately_before_dispatch() {
        for changed in [consumer_context(false, 7), consumer_context(true, 8)] {
            let queue = HookQueue::new(NonZeroUsize::new(4).unwrap()).unwrap();
            let current = consumer_context(true, 7);
            let mut consumer = initialized_consumer(&queue, current);
            for event in click_samples(10, 20)
                .into_iter()
                .chain([click_samples(30, 40)[0]])
            {
                assert_eq!(queue.try_push(event), Admission::Enqueued);
                consumer.poll_one(
                    &queue,
                    || current,
                    |_, _| empty(7),
                    |_| panic!("early intent"),
                );
            }
            assert_eq!(
                queue.try_push(click_samples(30, 40)[1]),
                Admission::Enqueued
            );
            let mut observations = 0;
            assert_eq!(
                consumer.poll_one(
                    &queue,
                    || {
                        observations += 1;
                        if observations < 3 { current } else { changed }
                    },
                    |_, _| empty(7),
                    |_| panic!("stale intent must not dispatch"),
                ),
                ConsumerOutcome::ContextChanged
            );
            assert_eq!(observations, 3);
            assert_eq!(queue.poll(), QueuePoll::Reset);
            assert!(consumer.recognizer.pressed.is_none());
            assert!(consumer.recognizer.candidate.is_none());
        }
    }

    #[test]
    fn consumer_owner_timeout_reset_clears_idle_history_and_pending_input() {
        let queue = HookQueue::new(NonZeroUsize::new(4).unwrap()).unwrap();
        let current = consumer_context(true, 7);
        let mut consumer = initialized_consumer(&queue, current);
        for event in click_samples(10, 20)
            .into_iter()
            .chain([click_samples(30, 40)[0]])
        {
            assert_eq!(queue.try_push(event), Admission::Enqueued);
            consumer.poll_one(
                &queue,
                || current,
                |_, _| empty(7),
                |_| panic!("early intent"),
            );
        }
        assert!(consumer.recognizer.pressed.is_some());
        assert!(consumer.recognizer.candidate.is_some());
        assert_eq!(
            queue.try_push(click_samples(30, 40)[1]),
            Admission::Enqueued
        );
        // The serialized owner signals timeout even with no further input.
        consumer.reset(&queue);
        assert!(consumer.recognizer.pressed.is_none());
        assert!(consumer.recognizer.candidate.is_none());
        assert_eq!(queue.try_push(click_samples(50, 60)[0]), Admission::Dropped);
        assert_eq!(
            consumer.poll_one(
                &queue,
                || current,
                |_, _| panic!("cancelled input must not hit test"),
                |_| panic!("cancelled input must not dispatch"),
            ),
            ConsumerOutcome::Reset
        );
        assert_eq!(queue.poll(), QueuePoll::Empty);
        let mut dispatched = Vec::new();
        for event in click_samples(50, 60)
            .into_iter()
            .chain(click_samples(70, 80))
        {
            assert_eq!(queue.try_push(event), Admission::Enqueued);
            consumer.poll_one(&queue, || current, |_, _| empty(7), |g| dispatched.push(g));
        }
        assert_eq!(dispatched, [7]);
    }

    #[test]
    fn consumer_initial_contention_defers_reset_without_processing_old_input() {
        let queue = HookQueue::new(NonZeroUsize::new(1).unwrap()).unwrap();
        assert_eq!(
            queue.try_push(click_samples(10, 20)[0]),
            Admission::Enqueued
        );
        let mut consumer = HookConsumer::new(DoubleClickSettings::new(100, 10, 10).unwrap());
        let guard = queue.ring.lock().unwrap();
        assert_eq!(
            consumer.poll_one(
                &queue,
                || consumer_context(true, 7),
                |_, _| panic!("contended validation must not hit test"),
                |_| panic!("contended validation must not dispatch"),
            ),
            ConsumerOutcome::Empty
        );
        drop(guard);
        assert_eq!(
            consumer.poll_one(
                &queue,
                || consumer_context(true, 7),
                |_, _| panic!("pre-validation sample must be discarded"),
                |_| panic!("pre-validation sample must not dispatch"),
            ),
            ConsumerOutcome::Reset
        );
        assert_eq!(queue.poll(), QueuePoll::Empty);
    }

    #[test]
    fn consumer_idle_context_change_cancels_history_before_later_samples() {
        let queue = HookQueue::new(NonZeroUsize::new(4).unwrap()).unwrap();
        let mut consumer = initialized_consumer(&queue, consumer_context(true, 7));
        for event in click_samples(10, 20) {
            assert_eq!(queue.try_push(event), Admission::Enqueued);
            consumer.poll_one(
                &queue,
                || consumer_context(true, 7),
                |_, _| empty(7),
                |_| panic!("early intent"),
            );
        }
        assert_eq!(
            consumer.poll_one(
                &queue,
                || consumer_context(true, 8),
                |_, _| panic!("idle poll must not hit test"),
                |_| panic!("idle poll must not dispatch"),
            ),
            ConsumerOutcome::ContextChanged
        );
        assert!(consumer.recognizer.candidate.is_none());
        assert_eq!(queue.try_push(click_samples(30, 40)[0]), Admission::Dropped);
        assert_eq!(queue.poll(), QueuePoll::Reset);
    }

    #[test]
    fn consumer_disabled_state_and_unknown_or_stale_hits_never_dispatch() {
        for hit in [DesktopHit::Unknown, DesktopHit::Occupied, empty(8)] {
            let queue = HookQueue::new(NonZeroUsize::new(4).unwrap()).unwrap();
            let context = std::cell::Cell::new(consumer_context(true, 7));
            let mut consumer = initialized_consumer(&queue, context.get());
            let mut dispatched = Vec::new();
            for event in click_samples(10, 20)
                .into_iter()
                .chain(click_samples(30, 40))
            {
                assert_eq!(queue.try_push(event), Admission::Enqueued);
                consume_one(&mut consumer, &queue, &context, |_, _| hit, &mut dispatched);
            }
            assert!(dispatched.is_empty());
        }

        let queue = HookQueue::new(NonZeroUsize::new(4).unwrap()).unwrap();
        let context = std::cell::Cell::new(consumer_context(false, 7));
        let mut consumer = initialized_consumer(&queue, context.get());
        let mut dispatched = Vec::new();
        for event in click_samples(10, 20) {
            assert_eq!(queue.try_push(event), Admission::Enqueued);
        }
        assert_eq!(
            consume_one(
                &mut consumer,
                &queue,
                &context,
                |_, _| empty(7),
                &mut dispatched,
            ),
            ConsumerOutcome::Disabled
        );
        assert_eq!(
            consume_one(
                &mut consumer,
                &queue,
                &context,
                |_, _| empty(7),
                &mut dispatched,
            ),
            ConsumerOutcome::Reset
        );
        assert!(dispatched.is_empty());
    }
}
