use orichalcum_macros::experimental_state_machine;

experimental_state_machine! {
    machine unreachable;
    initial Start;
    active Start;
    terminal Done;
    active Lost;
    transition finish: Start -> Done;
    transition stay_lost: Lost -> Lost;
}

fn main() {}
