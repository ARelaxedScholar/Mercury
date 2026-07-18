use orichalcum_macros::experimental_state_machine;

experimental_state_machine! {
    machine acyclic_cycle;
    initial A;
    active A;
    active B;
    transition forward: A -> B;
    transition backward: B -> A;
    policy acyclic;
}

fn main() {}
