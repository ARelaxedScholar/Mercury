#![cfg(feature = "experimental-graph")]

use std::convert::Infallible;

use orichalcum::experimental_state_machine;

experimental_state_machine! {
    machine public_preview;
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

#[test]
fn root_feature_reexports_an_executable_compiler_checked_machine() {
    let running = public_preview::Definition::start(Vec::<&'static str>::new());
    let outcome = running
        .dispatch(
            public_preview::RunningRoute::Complete,
            |events| {
                events.push("completed");
                Ok::<(), Infallible>(())
            },
            |_events| Ok::<(), Infallible>(()),
        )
        .expect("selected transition is infallible");

    let done = match outcome {
        public_preview::RunningOutcome::Complete(done) => done,
        public_preview::RunningOutcome::Cancel(_) => panic!("complete route was selected"),
    };

    assert_eq!(done.into_data(), ["completed"]);
}
