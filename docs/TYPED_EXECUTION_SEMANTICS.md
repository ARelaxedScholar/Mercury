# Orichalcum Typed Execution Semantics

## Status

This document is a pre-implementation design decision for typed node and transition
execution. It settles the failure-ownership and state-mutation questions identified in
`STATE_MACHINE_ROADMAP.md`.

The structural graph contract is defined separately in
`docs/STATE_MACHINE_VALIDITY.md`. A graph can be structurally valid while an execution
attempt still fails. This document defines what happens to an execution instance in
that case.

Normative terms such as **must**, **must not**, and **required** describe the contract
future typed and generated execution APIs are expected to satisfy. Illustrative type
shapes are not final public API proposals.

## Decision summary

Orichalcum adopts these execution rules:

1. A phase transition commits only after its transition effect returns success.
2. A failed node or transition remains in its source phase.
3. Ordinary returned failures give the caller ownership of the recoverable execution
   instance and the underlying error.
4. Domain-data mutations are eager and attempt-visible. Orichalcum does not promise
   automatic rollback.
5. Framework-owned control metadata is committed atomically at the transition boundary.
6. External side effects cannot be rolled back by the framework and require explicit
   retry/idempotency design.
7. Domain outcomes such as rejection or cancellation are successful state-machine
   results, not execution failures merely because their business meaning is negative.
8. Panics and dropped asynchronous futures are abnormal interruption, not ordinary
   `Result` failures.

The central tradeoff is deliberate: preserve state ownership and tell the truth about
partial mutation instead of requiring `D: Clone`, silently discarding state, or claiming
transactionality the framework cannot enforce.

## Current behavior and gap

The `0.5.0` typed API already has one sound control-state property:

- `Flow<P, D>::transition` relabels the state as the destination phase only after
  `Transition::advance` returns `Ok(())`.

However, `step`, `transition`, and branch execution consume their `Flow`. Their error
paths return only `N::Error`, `T::Error`, or `BranchExecuteError<R, E>`. Consequently:

- a failure does not produce a destination-phase value
- the source-phase value is also dropped
- mutations made before `Err` cannot be inspected through the typed API
- callers cannot correct data and retry without keeping some separate copy or shared
  handle
- a transition failure propagated by the branch builder loses the same state

The current API therefore prevents an invalid phase advance, but it does not provide
recoverable failure ownership. It also leaves mutation-on-failure behavior implicit.

The dynamic `NodeLogic`/`Flow` engine has a separate, older problem: several malformed
or failed situations are converted into logs, panics, empty values, or a `"default"`
route. That behavior is not the execution contract for the typed or future generated
engine.

## Execution model

An execution instance conceptually contains:

```text
Execution<P, D> {
    phase: P,
    data: D,
    metadata: ExecutionMetadata,
}
```

`P` represents framework-owned control state. `D` is user-owned workflow data.
`ExecutionMetadata` may eventually include definition identity, execution identity,
attempt counters, timestamps, and committed transition history.

The public handwritten API may continue to spell this concept `Flow<P, D>`. Generated
graph APIs may expose generated state-specific handles. Both must obey the same semantic
contract.

### Attempt boundaries

A node or transition invocation is one attempt. Each attempt has a stable identity when
telemetry, persistence, remote effects, or retry support is enabled.

An attempt has one of these outcomes:

- `Succeeded`: the operation returned normally with success
- `Failed`: the operation returned an ordinary typed error
- `Interrupted`: execution did not return normally because of panic, cancellation,
  process loss, or executor loss

Only `Succeeded` and `Failed` are ordinary `Result` outcomes. `Interrupted` requires an
executor or persistence mechanism capable of observing it; dropping a Rust future alone
cannot manufacture a returned value.

## Node semantics

A node executes within one phase and does not itself commit a phase change.

Conceptually:

```text
run_node(Execution<P, D>)
    -> NodeSucceeded<P, D, Route>
     | NodeFailed<P, D, Error>
```

### Node success

When a node returns `Ok(next)`:

- all mutations it made to `D` remain visible
- the execution remains in phase `P`
- the returned route or finish decision becomes the pending control decision
- no transition has committed yet

### Node failure

When a node returns `Err(error)`:

- the execution remains in phase `P`
- no route or finish decision is produced
- the failure owns the execution instance in phase `P`
- mutations made before returning `Err` remain present in `D`
- the caller may inspect, repair, persist, abandon, or explicitly retry the execution

A node error must not be converted into a default route, `Next::Finish`, log-only event,
or panic by the canonical typed executor.

## Transition semantics

A transition attempt has a source phase `P` and destination phase `Q`.

Conceptually:

```text
apply_transition(Execution<P, D>)
    -> TransitionSucceeded<Q, D>
     | TransitionFailed<P, D, Error>
```

### Transition success

The required order is:

1. begin an attempt while the execution is in source phase `P`
2. run the transition effect against the source-phase execution data
3. receive `Ok`
4. commit framework control state from `P` to `Q`
5. record the transition as committed
6. expose the destination-phase execution

The destination-phase value must not be observable before step 4.

### Transition failure

If the transition effect returns `Err(error)`:

- the phase commit does not occur
- the execution remains typed as source phase `P`
- the transition is recorded as attempted but not committed, when metadata is enabled
- the failure owns the source-phase execution and underlying error
- mutations made by the failed attempt remain present in `D`

The framework must never construct a destination-phase handle on this path.

### Phase atomicity

“Atomic transition” in Orichalcum refers only to framework-owned control state:

```text
phase before returned failure = P
phase after returned success = Q
```

It does not mean every mutation or external effect performed during the attempt is
transactionally rolled back.

## Domain-data mutation

### Eager mutation is the baseline

`StateNode` and `Transition` currently receive mutable access to `FlowState<P, D>`.
The baseline contract preserves that capability:

- mutations become part of the execution data when they occur
- returning `Err` does not undo them
- the recovered source-phase execution reveals their latest in-memory state

This rule avoids hidden cloning and works for non-`Clone` data, large data, resource
handles, and user-defined interior mutability.

### Why rollback is not the default

Generic automatic rollback would require at least one of:

- cloning or snapshotting arbitrary `D`
- requiring user-defined transaction hooks
- restricting data to a persistent or transactional representation
- pretending that in-memory restoration also reverses external effects

None is a truthful universal default. In particular, restoring a cloned `D` cannot
unsend an email, revoke an API request, or undo a database write already committed by an
external system.

### Explicit transactional behavior

Workflows that require rollback must opt into a mechanism whose scope is explicit, such
as:

- performing validation before mutation
- computing a change set and applying it only after fallible preparation succeeds
- using a database transaction owned by the effect
- using a user-provided snapshot/restore strategy
- using compensating transitions for already-committed external work

A future transactional adapter may exist, but its name and bounds must expose what it
actually guarantees. It must not change the baseline semantics silently.

### Mutation guidance

Effect authors should prefer this order when practical:

1. validate source data
2. perform fallible preparation
3. apply in-memory mutations
4. perform or confirm external work
5. return success

This reduces partial mutation but is guidance, not a compiler-enforced proof. Some
effects necessarily interleave work differently.

## Failure ownership

An ordinary execution failure must carry both error context and the recoverable
execution instance. A conceptual shape is:

```text
ExecutionFailure<P, D, E> {
    execution: Execution<P, D>,
    error: E,
    operation: Node(NodeId) | Transition(TransitionId),
    attempt: AttemptId,
}
```

The exact public type may differ, but it must support:

- borrowing the underlying error
- borrowing the recovered state and data
- consuming the failure into `(execution, error)`
- identifying whether a node or transition failed
- preserving the concrete source phase in typed APIs

No `Clone` bound is required. Ownership moves into success or failure exactly once.

### Recovery is not automatic retry

Returning the execution instance makes recovery possible; it does not automatically
retry the operation. The caller or an explicit retry policy decides whether retry is
safe.

A recovered transition failure is in the source phase, but the transition value itself
need not be recoverable. Generated definitions can refer to a registered transition
again. Handwritten one-shot transition objects may require the caller to construct a new
attempt explicitly.

### Branch failures

Framework-owned branch execution follows the same rule:

- an unhandled route returns the source-phase execution and the produced route
- missing finish handling returns the source-phase execution
- a selected transition failure returns the source-phase execution and transition error

In the future statically verified graph API, unhandled routes are definition-time errors
and cannot reach execution. They remain relevant to the current locally typed builder
until exhaustive definition-time route coverage replaces them.

Branch configuration errors should occur before execution whenever possible. If a
runtime builder consumes an execution-bearing value and then detects a duplicate route
or duplicate finish handler, its error must return that value rather than discard it.

## Error taxonomy

Orichalcum distinguishes four categories.

### Definition and configuration errors

These mean the machine or execution setup is invalid:

- invalid graph structure
- duplicate branch registration
- missing runtime route handler
- incompatible policies

Static definitions reject these during compilation. Dynamic definitions return
validation errors before execution. They are not node or transition failures.

### Execution failures

These are ordinary failures returned while attempting legal work:

- node-specific errors
- transition-specific errors
- infrastructure errors intentionally represented by an effect's error type

They return the recoverable execution instance as described above.

### Domain outcomes

These are successful business decisions represented in states, routes, or user-owned
outcome enums:

- approved or rejected
- completed or cancelled
- retry requested or manual review required
- success-terminal or failure-terminal domain states

The word “failure” in a domain state name does not make it an execution error. If the
machine legally transitions into `PaymentDeclined`, that transition succeeded.

### Abnormal interruption

Panics, process termination, task abortion, and dropped futures are not typed effect
errors. The framework must not silently convert them into a route or ordinary success.

Executors may catch and report panics at a boundary where Rust safety permits, but doing
so does not imply that user data or external effects are consistent. Panic recovery must
be labeled as interruption, not an ordinary node/transition error.

## External effects and retries

The framework cannot infer whether an external effect completed. A timeout can mean the
remote operation failed, succeeded, or remains in progress.

Therefore:

- Orichalcum does not promise exactly-once external effects
- retry policies must be explicit
- effect authors should use idempotency keys when the external system supports them
- attempt identity should remain stable and available to telemetry and effect adapters
- non-idempotent operations should require domain-specific reconciliation or
  compensation
- restoring in-memory state must not be described as external rollback

Persistence can strengthen crash recovery later, but it does not remove these distributed
systems constraints.

## Async execution and cancellation

Sync and async effects share all ordinary success and returned-failure semantics.
Cancellation adds one constraint: dropping a Rust future cannot return an
`ExecutionFailure`.

The contract therefore applies when an async operation completes with `Ok` or `Err`.
If it is cancelled or its future is dropped:

- it is `Interrupted`, not `Failed`
- no destination-phase commit may be recorded unless success was reached and committed
- partial in-memory mutation may exist if the execution instance is retained outside the
  future
- external completion may be unknown

The canonical async executor should keep durable execution ownership outside the
in-flight effect future where feasible, for example by borrowing a stable execution
record. Any API that cannot recover ownership after cancellation must document that
limitation and must not claim cancellation-safe recovery.

Cooperative cancellation that returns a typed error is an ordinary failure only when the
effect actually returns and the execution instance is recoverable. A domain transition
to a `Cancelled` state remains a successful domain outcome.

## Finish and terminality

The current `Next::Finish` means “stop this execution while remaining in the current
phase.” It does not prove that the current phase is a structurally terminal state.

For compatibility, the locally typed API may retain explicit finish handling. In the
future graph-defined API:

- reaching a strict terminal state ends execution structurally
- entering an absorbing state does not mean execution has ended
- an early stop/cancel signal must be modeled explicitly rather than silently equated
  with terminality

Finish handlers are infallible outcome constructors today. If fallible finalization is
introduced, it must be modeled as an effect with the same failure-ownership rules, not
hidden inside an outcome wrapper.

## Required API direction

The next typed API revision must make the following properties expressible:

```text
step:
  Execution<P, D>
    -> Result<Branch<P, D, R>, NodeFailure<P, D, E>>

transition:
  Execution<P, D>
    -> Result<Execution<Q, D>, TransitionFailure<P, D, E>>
```

These are semantic signatures, not settled Rust names.

The current error-only signatures may be preserved temporarily for compatibility, but
they must not remain the canonical recovery-capable path. A migration may use additive
methods first, followed by a breaking cleanup at an appropriate release boundary.

The design must not require a single global execution error enum. User effect errors
should remain typed, while framework wrappers add operation context and state ownership.

## Migration consequences for the current typed API

The additive implementation now provides:

- `step_recovering` and `transition_recovering`, returning `ExecutionFailure` with the
  concrete source-phase `Flow`
- `finish_recovering`, preserving the flow for transition failure, unhandled route, and
  missing finish handling
- `on_recovering` and `on_finish_recovering`, preserving execution-bearing builders on
  duplicate configuration
- mutation-before-error tests for nodes, direct transitions, and branch-selected
  transitions

The original methods remain as compatibility wrappers with their established error-only
surface. Callers that need inspection, repair, or retry should use the recovering forms.

Remaining implementation work includes:

1. Add stable operation and attempt identities once execution metadata exists.
2. Decide the release boundary at which recovering methods become the canonical names.
3. Apply the same explicit-error direction to the dynamic engine separately.
4. Add equivalent async typed effects after their shared adapter boundary exists.

Replacing the legacy signatures directly would be source-breaking. The additive strategy
avoids that break while making the stronger semantics available now.

The dynamic engine does not need to adopt these types immediately. Its panic, logging,
and implicit-default paths should be inventoried and migrated separately; the dynamic
escape hatch must eventually return explicit runtime execution errors rather than weaken
the typed contract.

## Conformance matrix

The execution-semantic test suite must include at least:

| Case | Required result |
| --- | --- |
| node succeeds without mutation | same phase, pending decision |
| node succeeds after mutation | same phase, mutated data, pending decision |
| node mutates then returns `Err` | source phase and mutated data returned with error |
| transition succeeds without mutation | destination phase returned |
| transition succeeds after mutation | destination phase with mutated data returned |
| transition mutates then returns `Err` | source phase and mutated data returned with error |
| selected branch transition fails | source phase and transition error returned |
| route is unhandled | source phase and route returned |
| finish handler is missing | source phase returned |
| domain rejection transition succeeds | rejection state, not execution error |
| panic occurs | no destination commit; classified as interruption if observed |
| async effect returns `Err` | same semantics as sync returned failure |
| async future is cancelled | interruption; no false returned-failure claim |

Compile-fail tests must continue proving that a failure path cannot be treated as a
destination-phase execution without explicitly obtaining a successful result.

## Telemetry and persistence implications

Telemetry should distinguish:

- attempt started
- node succeeded or failed
- transition effect succeeded or failed
- phase transition committed
- execution interrupted

A failed transition event names both source and intended destination but records the
source as the current phase. Only a commit event changes current phase.

Persistence should eventually use the same boundary. A durable implementation may need
an outbox, write-ahead record, or effect-specific reconciliation protocol, but those are
stronger execution modes rather than baseline in-memory guarantees.

## Decisions intentionally deferred

This document does not settle:

- final public error type names
- additive versus immediately breaking migration mechanics
- retry-policy syntax
- transactional adapter APIs
- persistence backend or event-log schema
- panic-catching policy for each executor
- effect timeouts
- compensation orchestration
- generated sync/async trait signatures

Those decisions must preserve the ownership and mutation contract established here.

## Acceptance criteria

This design is ready to guide implementation when:

- ordinary node and transition failures have unambiguous state ownership
- phase commit and domain-data mutation are clearly distinguished
- no automatic rollback or exactly-once guarantee is implied
- domain outcomes and execution failures are separate
- sync returned errors and async returned errors obey the same rules
- interruption and cancellation limitations are explicit
- current typed API migration gaps map to executable tests
