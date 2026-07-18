use orichalcum_macros::experimental_state_machine;

experimental_state_machine! {
    machine terminal_edge;
    initial Done;
    terminal Done;
    terminal Reopened;
    transition reopen: Done -> Reopened;
}

fn main() {}
