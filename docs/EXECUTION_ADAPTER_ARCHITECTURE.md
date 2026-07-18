# Orichalcum Execution-Adapter Architecture

## Status

This document defines how synchronous and asynchronous effects attach to one validated
graph artifact. It complements `TYPED_EXECUTION_SEMANTICS.md`; that document defines
success, failure, mutation, and interruption behavior, while this document defines the
architectural boundary that must preserve those semantics.

The type signatures below are conceptual. Final trait spelling remains an implementation
decision.

## Decision summary

Orichalcum will have one structural machine definition and multiple execution adapters.

```text
MachineDefinition
    -> validation
ValidatedMachine
    + effect bindings
    + execution data
    -> SyncExecutor or AsyncExecutor
```

Sync and async are capabilities of bound effects and executors. They are not different
graph types, different state categories, or different validity contracts.

## Separation of responsibilities

### Definition

The definition owns:

- state IDs and categories
- initial state
- transitions and finite route sets
- policies
- stable operation identities
- optional semantic contracts

It contains no running data, futures, locks, retry counters, or effect instances.

### Validated definition

`ValidatedMachine` proves that the definition satisfies its structural contract. It is
immutable and shareable across many executions.

Validation does not require node or transition effects to be available. An application
may inspect, visualize, serialize, or validate topology separately from deployment-time
effect binding.

### Effect bindings

Bindings associate stable node and transition identities with executable behavior.

Static generated APIs can make missing or type-incompatible bindings compiler errors.
Dynamic APIs validate binding completeness and compatibility before starting execution.

Bindings do not redefine topology. An effect cannot change its declared source,
destination, route, or state category.

### Execution instance

An execution instance owns:

- current state
- domain data
- execution and attempt identity
- committed runtime metadata

It refers to one validated definition. It does not own or mutate that definition.

### Executor

An executor selects the operation allowed by the current state, invokes the bound effect,
and commits framework control state according to `TYPED_EXECUTION_SEMANTICS.md`.

The executor owns scheduling mechanics, not domain decisions.

## Effect model

The structural layer recognizes two execution roles.

### State/node effect

A state-local effect examines or mutates data and returns a declared control decision:

```text
NodeEffect<P, D, Routes, E>
    D at P -> Result<RouteDecision<Routes>, E>
```

It does not directly relabel the execution phase. Its decision selects a declared
transition or an explicitly modeled stop behavior.

### Transition effect

A transition effect performs validation or work associated with one declared edge:

```text
TransitionEffect<P, Q, D, E>
    D at P -> Result<(), E>
```

Only the executor commits `P -> Q`, and only after the effect succeeds.

Some domains may need effects only on states, only on transitions, or on both. The graph
model should allow an explicit no-op effect without making topology depend on the choice.

## Sync adapter

A conceptual synchronous binding is:

```rust,ignore
trait SyncTransitionEffect<P, Q, D> {
    type Error;

    fn apply(&self, state: &mut ExecutionState<P, D>)
        -> Result<(), Self::Error>;
}
```

The sync executor returns success or a failure that owns the recoverable source-phase
execution. It must not translate errors into default routes or log-only continuation.

## Async adapter

A conceptual asynchronous binding is:

```rust,ignore
trait AsyncTransitionEffect<P, Q, D> {
    type Error;
    type Future<'a>: Future<Output = Result<(), Self::Error>>
    where
        Self: 'a,
        D: 'a;

    fn apply<'a>(&'a self, state: &'a mut ExecutionState<P, D>)
        -> Self::Future<'a>;
}
```

This illustrates the desired capability without selecting GAT futures, return-position
`impl Trait`, boxed futures, or `async_trait` as the final spelling.

The async executor follows the same returned `Ok`/`Err` phase-commit contract. Dropped
futures and task abortion remain interruption, as defined in the execution-semantics
document.

## Adapting synchronous effects into async execution

An async executor may run a synchronous effect only through an explicit adapter.

Two cases must remain distinct:

- short, nonblocking sync work may execute inline through a named adapter
- blocking work must use a configured blocking pool or caller-provided scheduler

The framework must not assume that every synchronous effect is safe to poll on an async
runtime thread. Adapter selection is deployment behavior and does not alter the graph.

The reverse adaptation is not generally available: a sync executor cannot run arbitrary
async effects without choosing and owning an async runtime. Orichalcum should not hide a
global `block_on` inside the canonical sync executor.

## Binding completeness

Graph validity and deployment completeness are separate checks.

### Graph validity

Answers whether states, transitions, routes, and policies form a legal machine.

### Binding validity

Answers whether a selected executor has every effect it needs and whether those effects
are compatible with the declared operation types.

For static definitions, generated constructors should make incomplete bindings
unrepresentable where practical. For dynamic definitions, startup validation returns a
structured binding report before an execution instance is created.

A valid graph with missing production bindings is not executable, but it remains a valid
graph descriptor.

## Typed and erased boundaries

Static generated execution should preserve concrete phase, route, data, and effect error
types at the user-facing boundary.

Runtime-defined graphs necessarily erase some types behind stable IDs and trait objects.
That erasure belongs in the dynamic binding/executor layer, not in the shared structural
validator.

Both modes still share:

- state and transition identities
- state categories
- route coverage rules
- policy checks
- phase-commit semantics
- failure ownership requirements

Type erasure is not permission to weaken validation.

## Error composition

The executor adds context around, rather than replacing, user effect errors.

Conceptually:

```text
ExecutionFailure {
    source_execution,
    operation_id,
    attempt_id,
    cause: user effect error,
}
```

Static APIs should retain the concrete effect error type. Dynamic APIs may use an erased
error source with stable framework context.

Definition errors, missing bindings, domain outcomes, returned effect failures, and
interruptions remain distinct categories.

## Route execution

The generated graph API owns route-to-transition dispatch:

1. invoke the current state's node effect
2. receive one value from its declared finite route set
3. select the exactly-one transition validated for that route
4. invoke that transition's bound effect
5. commit the destination state on success

No runtime branch builder is necessary for a statically verified graph. The current
typed branch builder remains a local-safety compatibility layer until generated dispatch
exists.

Dynamic execution performs the same sequence through validated route tables. An
unhandled route after successful definition validation is an internal invariant failure,
not an expected user execution error.

## Concurrency and scheduling

Graph structure does not imply a scheduler. The first generated executor should remain
single-execution and sequential unless the definition explicitly introduces concurrency
semantics later.

Async support means effects may suspend; it does not automatically mean transitions run
in parallel.

Future parallel regions, joins, or races require additional structural vocabulary and
validity rules. They should not be inferred from multiple outgoing edges.

## Persistence boundary

Persistence observes the same attempt and phase-commit events in both executors:

```text
attempt started at P
effect returned success or failure
transition committed P -> Q
```

An async executor may have more interruption points, but the durable record format should
not become an async-specific graph schema.

## Telemetry boundary

Telemetry fields should be structural and executor-neutral:

- machine definition ID
- execution ID
- state ID
- operation and transition IDs
- attempt ID
- route ID or domain outcome
- effect mode (`sync`, `async`, or adapted sync)
- returned success/failure or observed interruption
- committed destination, if any

Executor-specific timing or runtime metadata may be additional fields.

## Why there is no second async graph API

Duplicating builders or definitions would create avoidable drift:

- categories could acquire different meanings
- route coverage could differ
- diagnostics could diverge
- static and dynamic descriptors could become executor-shaped

Only effect binding and invocation differ. The graph artifact, validator, generated
state identities, and transition tables remain one model.

## Initial implementation order

1. Stabilize and extract the span-neutral definition core.
2. Add a static descriptor representation produced from `ValidatedMachine`.
3. Define executor-neutral operation IDs and binding requirements.
4. Implement the sync transition boundary with recoverable failure ownership.
5. Add async effect traits against the same boundary.
6. Test returned-error parity between sync and async adapters.
7. Add interruption and cancellation tests where the chosen runtime permits observation.
8. Generate route dispatch only after the macro front end validates exact coverage.

This avoids building a second async branch hierarchy that the generated graph path would
immediately supersede.

## Prototype status

The experimental macro now implements the smallest synchronous vertical slice:

- call-site closures act as transition-effect bindings
- direct effects return destination-phase execution only on success
- routed states generate exhaustive dispatch and typed destination outcomes
- failures preserve the mutated source-phase execution and concrete effect error

This validates the semantic boundary but does not yet implement the recommended reusable
binding artifact. The next executor spike should move effects out of each dispatch call
and validate binding completeness once per definition/deployment.

## Required conformance tests

Equivalent sync and async effects must demonstrate:

- identical successful source-to-destination commit
- identical source-phase ownership on returned error
- identical visibility of mutation-before-error
- identical route selection and transition identity
- identical domain outcome classification
- identical structural and binding validation

Async-only tests additionally cover future cancellation and blocking-effect adapters.

## Decisions intentionally deferred

This document does not select:

- final effect trait names
- GAT futures versus boxed futures versus `async_trait`
- dependency-injection syntax
- executor ownership and borrowing ergonomics
- retry middleware shape
- persistence implementation
- parallel execution constructs

Those choices must preserve one graph artifact and the execution semantics already
settled.

## Acceptance criteria

The adapter architecture is settled when:

- definition, bindings, execution instance, and executor have distinct responsibilities
- graph validity is independent of sync or async effects
- both executors share phase-commit and failure-ownership semantics
- generated route dispatch replaces duplicated branch-builder logic
- sync-to-async adaptation is explicit and blocking-safe
- async cancellation is not misrepresented as an ordinary returned failure
