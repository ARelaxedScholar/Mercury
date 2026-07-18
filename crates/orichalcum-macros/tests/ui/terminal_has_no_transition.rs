use orichalcum_macros::experimental_state_machine;

experimental_state_machine! {
    machine terminal_execution;
    initial Running;
    active Running;
    terminal Done;
    transition complete: Running -> Done;
}

fn main() {
    let done = terminal_execution::Definition::start(())
        .complete(|_| Ok::<(), ()>(()))
        .unwrap();
    let _ = done.complete(|_| Ok::<(), ()>(()));
}
