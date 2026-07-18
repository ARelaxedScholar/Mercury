use std::convert::Infallible;

use orichalcum_macros::experimental_state_machine;

experimental_state_machine! {
    machine review_flow;
    initial Draft;
    active Draft;
    terminal Done;
    transition finish: Draft -> Done;
}

fn main() {
    assert_eq!(review_flow::Definition::STATES.len(), 2);
    assert_eq!(review_flow::Definition::TRANSITIONS.len(), 1);
    let _draft = review_flow::Draft;
    let _done = review_flow::Done;
    let done = review_flow::Definition::start("document")
        .finish(|_| Ok::<(), Infallible>(()))
        .expect("transition effect is infallible");
    assert_eq!(done.into_data(), "document");
}
