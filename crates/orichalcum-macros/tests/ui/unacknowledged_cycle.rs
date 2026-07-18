use orichalcum_macros::experimental_state_machine;

experimental_state_machine! {
    machine unacknowledged_cycle;
    initial Running;
    active Running;
    transition tick: Running -> Running;
    policy cycles_explicit;
}

fn main() {}
