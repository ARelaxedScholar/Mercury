use orichalcum_macros::experimental_state_machine;

experimental_state_machine! {
    machine absorbing_escape;
    initial Running;
    active Running;
    absorbing Cancelled;
    terminal Reopened;
    transition cancel: Running -> Cancelled;
    transition reopen: Cancelled -> Reopened;
}

fn main() {}
