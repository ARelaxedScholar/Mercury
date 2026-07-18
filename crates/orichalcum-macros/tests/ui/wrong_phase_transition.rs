use orichalcum_macros::experimental_state_machine;

experimental_state_machine! {
    machine wrong_phase;
    initial Draft;
    active Draft;
    active Review;
    terminal Approved;
    transition submit: Draft -> Review;
    transition approve: Review -> Approved;
}

fn main() {
    let draft = wrong_phase::Definition::start(());
    let _approved = draft.approve(|_| Ok::<(), ()>(()));
}
