use orichalcum_macros::experimental_state_machine;

experimental_state_machine! {
    machine recoverable;
    initial Draft;
    active Draft;
    terminal Review;
    transition submit: Draft -> Review;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rejected;

fn main() {
    let draft = recoverable::Definition::start(vec!["created"]);
    let failure = draft
        .submit(|events| {
            events.push("attempted");
            Err(Rejected)
        })
        .expect_err("effect intentionally fails");

    assert_eq!(failure.transition(), "submit");
    assert_eq!(failure.error(), &Rejected);
    assert_eq!(failure.execution().data(), &["created", "attempted"]);
    let (draft, error): (recoverable::Execution<recoverable::Draft, _>, _) =
        failure.into_parts();
    assert_eq!(error, Rejected);
    assert_eq!(draft.into_data(), ["created", "attempted"]);
}
