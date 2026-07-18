//! A phase-typed lifecycle with a retry cycle, terminal completion, absorbing
//! cancellation, and explicit handling for every route.
//!
//! The current typed API proves local phase legality. Route coverage is still checked by
//! the runtime branch builder; the future graph-definition macro will make the same
//! topology compiler-verified as a whole.

use std::convert::Infallible;

use orichalcum::typed::{Flow, FlowState, Next, StateNode, Transition};

#[derive(Debug)]
struct Ready;
#[derive(Debug)]
struct Running;
#[derive(Debug)]
struct Backoff;
#[derive(Debug)]
struct Completed;
#[derive(Debug)]
struct Cancelled;

#[derive(Debug)]
struct LifecycleData {
    attempts: u8,
    cancel_requested: bool,
    events: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkDecision {
    Complete,
    Retry,
    Cancel,
}

struct StartWork;

impl Transition<Ready, LifecycleData> for StartWork {
    type NextPhase = Running;
    type Error = Infallible;

    fn advance(&self, state: &mut FlowState<Ready, LifecycleData>) -> Result<(), Self::Error> {
        state.data_mut().events.push("started");
        Ok(())
    }
}

struct Work;

impl StateNode<Running, LifecycleData> for Work {
    type Route = WorkDecision;
    type Error = Infallible;

    fn run(
        &self,
        state: &mut FlowState<Running, LifecycleData>,
    ) -> Result<Next<Self::Route>, Self::Error> {
        if state.data().cancel_requested {
            return Ok(Next::Route(WorkDecision::Cancel));
        }

        state.data_mut().attempts += 1;
        if state.data().attempts == 1 {
            Ok(Next::Route(WorkDecision::Retry))
        } else {
            Ok(Next::Route(WorkDecision::Complete))
        }
    }
}

struct CompleteWork;

impl Transition<Running, LifecycleData> for CompleteWork {
    type NextPhase = Completed;
    type Error = Infallible;

    fn advance(&self, state: &mut FlowState<Running, LifecycleData>) -> Result<(), Self::Error> {
        state.data_mut().events.push("completed");
        Ok(())
    }
}

struct ScheduleRetry;

impl Transition<Running, LifecycleData> for ScheduleRetry {
    type NextPhase = Backoff;
    type Error = Infallible;

    fn advance(&self, state: &mut FlowState<Running, LifecycleData>) -> Result<(), Self::Error> {
        state.data_mut().events.push("retry scheduled");
        Ok(())
    }
}

struct Retry;

impl Transition<Backoff, LifecycleData> for Retry {
    type NextPhase = Running;
    type Error = Infallible;

    fn advance(&self, state: &mut FlowState<Backoff, LifecycleData>) -> Result<(), Self::Error> {
        state.data_mut().events.push("retrying");
        Ok(())
    }
}

struct CancelWork;

impl Transition<Running, LifecycleData> for CancelWork {
    type NextPhase = Cancelled;
    type Error = Infallible;

    fn advance(&self, state: &mut FlowState<Running, LifecycleData>) -> Result<(), Self::Error> {
        state.data_mut().events.push("cancelled");
        Ok(())
    }
}

/// A self-transition models the local behavior expected of an absorbing state.
struct ObserveCancellation;

impl Transition<Cancelled, LifecycleData> for ObserveCancellation {
    type NextPhase = Cancelled;
    type Error = Infallible;

    fn advance(&self, state: &mut FlowState<Cancelled, LifecycleData>) -> Result<(), Self::Error> {
        state.data_mut().events.push("cancellation observed");
        Ok(())
    }
}

#[derive(Debug)]
enum WorkOutcome {
    Completed(Flow<Completed, LifecycleData>),
    Backoff(Flow<Backoff, LifecycleData>),
    Cancelled(Flow<Cancelled, LifecycleData>),
    Paused(Flow<Running, LifecycleData>),
}

fn run_once(flow: Flow<Running, LifecycleData>) -> WorkOutcome {
    flow.step(&Work)
        .expect("work node is infallible")
        .branch::<WorkOutcome>()
        .on(WorkDecision::Complete, CompleteWork, WorkOutcome::Completed)
        .expect("complete route is unique")
        .on(WorkDecision::Retry, ScheduleRetry, WorkOutcome::Backoff)
        .expect("retry route is unique")
        .on(WorkDecision::Cancel, CancelWork, WorkOutcome::Cancelled)
        .expect("cancel route is unique")
        .on_finish(WorkOutcome::Paused)
        .expect("finish handling is configured once")
        .finish()
        .expect("every produced decision has a handler")
}

fn main() {
    let running = Flow::<Ready, _>::new(LifecycleData {
        attempts: 0,
        cancel_requested: false,
        events: Vec::new(),
    })
    .transition(StartWork)
    .expect("start transition is infallible");

    let backoff = match run_once(running) {
        WorkOutcome::Backoff(flow) => flow,
        WorkOutcome::Paused(flow) => panic!(
            "first attempt unexpectedly paused after {} attempts",
            flow.data().attempts
        ),
        outcome => panic!("first attempt should request retry, got {outcome:?}"),
    };
    let running = backoff
        .transition(Retry)
        .expect("retry transition is infallible");
    let completed = match run_once(running) {
        WorkOutcome::Completed(flow) => flow,
        outcome => panic!("second attempt should complete, got {outcome:?}"),
    };

    // `Completed` intentionally has no node or outgoing transition implementation. The
    // future graph definition will declare and verify that terminal category centrally.
    assert_eq!(completed.data().attempts, 2);
    assert_eq!(
        completed.data().events,
        ["started", "retry scheduled", "retrying", "completed"]
    );

    let running = Flow::<Ready, _>::new(LifecycleData {
        attempts: 0,
        cancel_requested: true,
        events: Vec::new(),
    })
    .transition(StartWork)
    .expect("start transition is infallible");
    let cancelled = match run_once(running) {
        WorkOutcome::Cancelled(flow) => flow,
        outcome => panic!("cancel request should enter cancellation, got {outcome:?}"),
    };
    let cancelled = cancelled
        .transition(ObserveCancellation)
        .expect("absorbing self-transition is infallible");

    assert_eq!(
        cancelled.data().events,
        ["started", "cancelled", "cancellation observed"]
    );
}
