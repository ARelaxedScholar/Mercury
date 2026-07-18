#[test]
fn validates_complete_graph_during_expansion() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/valid.rs");
    tests.pass("tests/ui/valid_full.rs");
    tests.pass("tests/ui/valid_recovery.rs");
    tests.compile_fail("tests/ui/missing_route.rs");
    tests.compile_fail("tests/ui/absorbing_escape.rs");
    tests.compile_fail("tests/ui/acyclic_cycle.rs");
    tests.compile_fail("tests/ui/unacknowledged_cycle.rs");
    tests.compile_fail("tests/ui/wrong_phase_transition.rs");
    tests.compile_fail("tests/ui/terminal_has_no_transition.rs");
    tests.compile_fail("tests/ui/unknown_destination.rs");
    tests.compile_fail("tests/ui/terminal_edge.rs");
    tests.compile_fail("tests/ui/unreachable.rs");
}
