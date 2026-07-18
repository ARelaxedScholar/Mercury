# Orichalcum graph-definition core

`orichalcum-definition` is the span-neutral graph model and validation engine used by
Orichalcum's compiler-verified state-machine macro.

This is a published implementation crate so the procedural macro can be distributed
through crates.io. Most users should depend only on `orichalcum` and enable its
`experimental-graph` feature.

The crate validates complete graph definitions, including initial-state identity,
reachability, state-category constraints, finite route coverage, terminal reachability,
and cycle policies. Its diagnostics have stable codes, while its Rust API is not yet a
stable public integration surface during Orichalcum's 0.x series.
