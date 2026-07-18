use std::convert::Infallible;

use orichalcum_macros::experimental_state_machine;

experimental_state_machine! {
    machine lifecycle;
    initial Running;
    active Running;
    routes Running { Complete, Cancel };
    terminal Done;
    absorbing Cancelled;
    transition complete: Running -> Done on Complete;
    transition cancel: Running -> Cancelled on Cancel;
    transition observe: Cancelled -> Cancelled cycle;
    policy persistent;
    policy cycles_explicit;
}

fn main() {
    assert_eq!(lifecycle::Definition::INITIAL, "Running");
    assert_eq!(lifecycle::Definition::STATES.len(), 3);
    assert_eq!(lifecycle::Definition::ROUTES[0].1, ["Complete", "Cancel"]);
    assert_eq!(lifecycle::Definition::TRANSITIONS.len(), 3);
    assert_eq!(lifecycle::Definition::POLICIES, ["persistent", "cycles_explicit"]);

    let running = lifecycle::Definition::start(Vec::<&'static str>::new());
    let outcome = running
        .dispatch(
            lifecycle::RunningRoute::Complete,
            |events| {
                events.push("completed");
                Ok::<(), Infallible>(())
            },
            |_events| Ok::<(), Infallible>(()),
        )
        .expect("selected transition effect is infallible");
    let done = match outcome {
        lifecycle::RunningOutcome::Complete(done) => done,
        lifecycle::RunningOutcome::Cancel(_) => panic!("complete route was selected"),
    };
    assert_eq!(done.data(), &["completed"]);

    let running = lifecycle::Definition::start(Vec::<&'static str>::new());
    let outcome = running
        .dispatch(
            lifecycle::RunningRoute::Cancel,
            |_events| Ok::<(), Infallible>(()),
            |events| {
                events.push("cancelled");
                Ok::<(), Infallible>(())
            },
        )
        .expect("selected transition effect is infallible");
    let cancelled = match outcome {
        lifecycle::RunningOutcome::Cancel(cancelled) => cancelled,
        lifecycle::RunningOutcome::Complete(_) => panic!("cancel route was selected"),
    };
    let cancelled = cancelled
        .observe(|events| {
            events.push("observed");
            Ok::<(), Infallible>(())
        })
        .expect("absorbing effect is infallible");
    assert_eq!(cancelled.data(), &["cancelled", "observed"]);
}
