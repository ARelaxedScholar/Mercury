use orichalcum_macros::experimental_state_machine;

experimental_state_machine! {
    machine missing_route;
    initial Review;
    active Review;
    routes Review { Approve, RequestChanges };
    terminal Approved;
    transition approve: Review -> Approved on Approve;
}

fn main() {}
