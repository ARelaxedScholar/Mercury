use orichalcum_macros::experimental_state_machine;

experimental_state_machine! {
    machine unknown_destination;
    initial Start;
    active Start;
    terminal Done;
    transition leave: Start -> Missing;
}

fn main() {}
