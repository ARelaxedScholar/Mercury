# Orichalcum experimental state-machine macro

This published implementation crate provides Orichalcum's end-to-end preview of
compiler-verified state-machine definitions. Most users should enable the
`experimental-graph` feature on `orichalcum` and import the macro from there. Its syntax
and generated API are not stable during the 0.x series.

## Current slice

```rust,ignore
experimental_state_machine! {
    machine lifecycle;
    initial Running;
    active Running;
    routes Running { Complete, Cancel };
    terminal Done;
    absorbing Cancelled;

    transition complete: Running -> Done on Complete;
    transition cancel: Running -> Cancelled on Cancel;
    transition observe: Cancelled -> Cancelled cycle;

    policy persistent;
    policy cycles_explicit;
}
```

Macro expansion:

- validates the complete graph using `orichalcum-definition`
- rejects invalid initial states, endpoints, reachability, state categories, route
  coverage, and policies with stable diagnostic codes
- generates phase marker types and an inspectable static descriptor
- generates fallible direct-transition methods only for their source phase
- generates exhaustive route and outcome enums for routed states
- returns source-phase ownership and mutation-before-error data when an effect fails
- generates no outgoing methods for strict terminal states

Effects are supplied at the call site in this prototype:

```rust,ignore
let done = lifecycle::Definition::start(data)
    .dispatch(
        lifecycle::RunningRoute::Complete,
        |data| complete(data),
        |data| cancel(data),
    )?;
```

## Deliberate limitations

- effect bindings are call-site closures, not a reusable validated binding artifact
- routed transitions share one error type per dispatch call
- node effects do not yet produce the route value
- async effects are not supported
- all phases carry one domain-data type
- guards, persistence, retries, dependency injection, and telemetry are not generated
- declarations are currently order-sensitive
- generated-name collision diagnostics are not implemented
- the root re-export requires Orichalcum's `experimental-graph` feature

These constraints keep the spike small enough to evaluate graph diagnostics, generated
phase legality, typed route dispatch, and recoverable effect failure together. They are
not promises about the public API.
