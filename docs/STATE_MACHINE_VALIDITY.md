# Orichalcum State-Machine Validity Specification

## Status

This document is a pre-implementation design specification for the compiler-verified
state-machine model described in `STATE_MACHINE_ROADMAP.md`.

It defines what Orichalcum means by a structurally valid state machine. It intentionally
does not choose a public macro syntax, generated API shape, or executor implementation.
Those decisions must conform to this specification rather than define validity
implicitly through whichever API is implemented first.

Normative terms such as **must**, **must not**, **required**, and **invalid** describe
rules future implementations are expected to enforce. Sections explicitly labeled as
non-normative provide rationale or examples.

## Objective

For a workflow whose complete topology is declared statically, invalid general
structure must be rejected during compilation. For a workflow whose topology is built
at runtime, the same rules must be evaluated before execution and returned as structured
validation errors.

Compiler verification covers the declared graph and selected graph policies. It does
not prove arbitrary Rust business logic correct, guarantee that an effect succeeds, or
infer domain meaning that the workflow author did not declare.

## Scope

This specification covers:

- state identity and state categories
- initial-state declaration
- transition sources, destinations, and triggers
- incoming and outgoing edge requirements
- reachability from the initial state
- strict terminal and absorbing-state behavior
- route-to-transition coverage
- optional liveness and cycle policies
- deterministic validation order
- diagnostic structure and quality
- parity between compile-time and runtime validation

This specification does not yet cover:

- semantic input/output compatibility between nodes
- execution error types or recovery semantics (see `TYPED_EXECUTION_SEMANTICS.md`)
- mutation rollback when an effect fails (see `TYPED_EXECUTION_SEMANTICS.md`)
- concurrency, scheduling, or distributed execution
- persistence and restoration of running instances
- authorization to perform transition effects
- proof that arbitrary guards are satisfiable
- proof that execution will terminate in finite time

Those concerns may build on a valid graph but are not part of graph validity itself.

## Abstract graph model

A state-machine definition lowers to an implementation-neutral graph:

```text
Machine = (States, Transitions, Initial, Policies)
```

### State

Each state has:

```text
State {
    id: StateId,
    category: Active | Terminal | Absorbing,
    declared_at: SourceLocation,
}
```

`StateId` must be unique within one machine. A state ID represents structural identity;
display names, descriptions, or generated Rust type names do not create additional
states.

State categories have these meanings:

- `Active`: execution may leave the state and the state must declare outgoing behavior.
- `Terminal`: execution has structurally ended and the state has no outgoing transitions.
- `Absorbing`: execution may continue, but every outgoing transition returns to the same
  state.

Success, failure, cancellation, approval, rejection, and similar meanings remain domain
concepts. A terminal state may represent any of them.

### Transition

Each transition has:

```text
Transition {
    id: TransitionId,
    source: StateId,
    destination: StateId,
    trigger: Direct | Route(RouteId),
    declared_at: SourceLocation,
}
```

`TransitionId` must be unique within one machine. The executor or effect associated with
a transition is intentionally absent from the structural model.

A `Direct` transition is selected without a route value. A `Route(RouteId)` transition
handles one declared route produced while the machine is in its source state. Route IDs
are scoped to their source state, so two states may use the same route name without a
collision.

Two transitions from the same source must not claim the same trigger. This prevents both
duplicate route handlers and ambiguous direct transitions.

### Initial state

`Initial` identifies exactly one declared state. Initiality is a property of the machine
definition, not an implicit consequence of declaration order.

The initial state may be active, terminal, or absorbing. A single-state terminal machine
is therefore valid.

### Route set

When a state branches on a finite route type, its definition exposes the complete set of
declared routes:

```text
Routes(source_state) = {route_1, route_2, ...}
```

The static definition mechanism must make this finite set inspectable. A design that can
only observe route values as they occur at runtime cannot provide compile-time exhaustive
route coverage and does not satisfy the strategic target.

### Policies

Policies add stronger constraints to the unconditional structural rules. They never
weaken those rules.

## Unconditional structural invariants

Every machine must satisfy all invariants in this section, regardless of its selected
policies.

### V001 — The machine is nonempty

At least one state must be declared.

### V002 — Exactly one initial state is declared

Missing and multiply declared initial states are invalid. Declaration order must not be
used as an implicit fallback. The initial declaration must identify a state declared in
the same machine.

### V003 — State identities are unique

No two state declarations may use the same `StateId`.

### V004 — Transition identities are unique

No two transition declarations may use the same `TransitionId`.

### V005 — Transition endpoints exist

Every transition source and destination must refer to a declared state in the same
machine.

### V006 — Transition triggers are unambiguous

For a given source state, no trigger may select more than one transition.

Consequences include:

- a route may be handled at most once from a state
- a state may have at most one unqualified `Direct` transition
- parallel edges are allowed only when they have distinct route triggers

### V007 — Every declared state is reachable

Every state must be reachable by a directed path beginning at the initial state. The
initial state is reachable by the empty path.

This rule rejects disconnected states and disconnected cycles even when every state in
the disconnected component has an incoming edge.

### V008 — Every non-initial state has an incoming transition

Every state other than the initial state must have at least one incoming transition.

This follows logically from reachability, but remains a named invariant because it
supports a more direct diagnostic than a general reachability error when a state has no
incoming edges at all.

### V009 — Every active state has outgoing behavior

An active state must have at least one outgoing transition. A self-transition counts as
outgoing behavior.

Whether a self-transition-only active component is acceptable under a selected liveness
policy is a separate policy question.

### V010 — Strict terminal states have no outgoing transitions

A terminal state must not be the source of any transition, including a self-transition.

### V011 — Absorbing states cannot be exited

Every transition whose source is absorbing must have that same state as its destination.

An absorbing state must declare at least one self-transition. Without one it is
structurally indistinguishable from a strict terminal state and should be declared
terminal instead.

### V012 — Declared routes are handled exactly once

For every state with a declared finite route set:

```text
declared routes = routed transition triggers
```

Therefore:

- every declared route has one handler
- no route has multiple handlers
- no transition handles an undeclared route

A finish or stop signal is not silently treated as a route. If the future execution
model retains such a signal, its structural meaning must be declared separately.

## Optional graph policies

Policies are compile-time checked for static definitions and runtime checked for dynamic
definitions. Policy names below are conceptual; final public spelling may differ.

### P001 — `must_reach_terminal`

At least one terminal state must exist, and every reachable nonterminal state must have
a path to a terminal state.

An absorbing state does not satisfy terminal reachability because execution continues
structurally within it.

This is a possibility guarantee, not a finite-time termination proof. Guards, effect
failures, scheduling, or repeated route choices may still prevent a particular execution
from taking the terminating path.

### P002 — `acyclic`

The graph must contain no directed cycle. A self-transition is a cycle, so absorbing
states are incompatible with this policy.

### P003 — `cycles_explicit`

Every transition participating in a directed cycle must belong to a cycle explicitly
acknowledged by the definition. The future syntax may mark transitions, states, or named
cycle groups, but accidental cycles must be rejected.

### P004 — `persistent`

The definition explicitly states that terminal reachability is not required. Closed
nonterminal strongly connected components are allowed.

`persistent` does not waive reachability, route coverage, state-category rules, or any
other unconditional invariant.

### Policy compatibility

The initial policy compatibility rules are:

| Policy combination | Validity |
| --- | --- |
| `must_reach_terminal` + `acyclic` | Allowed |
| `must_reach_terminal` + `cycles_explicit` | Allowed |
| `persistent` + `cycles_explicit` | Allowed |
| `persistent` + `acyclic` | Allowed |
| `persistent` + `must_reach_terminal` | Invalid as contradictory intent |

An absorbing state with `must_reach_terminal` is valid only if the absorbing state is
unreachable, but V007 already rejects unreachable states. Consequently, a reachable
absorbing state and `must_reach_terminal` are incompatible in a valid machine.

## Validation order

Validation must be deterministic and should avoid diagnostics that are merely downstream
effects of an earlier malformed declaration.

The required validation phases are:

1. Parse and lower the definition into the abstract graph.
2. Validate machine nonemptiness and identifier uniqueness.
3. Validate the initial-state declaration.
4. Validate transition endpoint references.
5. Validate trigger uniqueness and route declarations.
6. Validate state-category edge rules and local incoming/outgoing requirements.
7. Validate reachability from the initial state using every endpoint-valid declared
   transition.
8. Validate exhaustive route coverage.
9. Validate selected graph policies.

Graph analyses that depend on a unique initial state must not run when V002 fails.
Unknown transition endpoints must not be inserted as synthetic states merely to continue
analysis.

Reachability includes a declared transition even when that transition violates a state
category rule. For example, the destination of an illegal terminal-state transition is
still reachable through the declared graph. This reports the root `SM011` violation
without adding a misleading `SM014`. Trigger ambiguity likewise does not remove an
otherwise endpoint-valid edge from reachability analysis.

An unreachable non-initial state with no incoming transition receives the more specific
`SM009`; `SM014` is suppressed for that state. An unreachable state that does have an
incoming transition, including a member of a disconnected component, receives `SM014`.

Policy analysis runs only after the unconditional structural phases succeed. A malformed
graph must be repaired before liveness or cycle-policy diagnostics are meaningful.

An implementation may report multiple independent errors from one phase, but diagnostic
ordering must remain stable for the same definition.

## Diagnostic contract

Compile-time and runtime validators must use the same diagnostic codes and core messages.
Presentation may differ: a macro should attach errors to source spans, while a runtime
validator should return structured locations or declaration IDs.

Each diagnostic contains:

```text
Diagnostic {
    code: DiagnosticCode,
    severity: Error | Warning,
    message: String,
    primary: DefinitionLocation,
    related: [DefinitionLocation],
    witness: Optional<GraphWitness>,
}
```

Validity violations are errors. Warnings may provide advice but must not be required to
determine whether a graph is valid.

### Stable diagnostic codes

| Code | Rule | Required primary information |
| --- | --- | --- |
| `SM001` | V001 | machine definition |
| `SM002` | V002, missing initial | machine definition |
| `SM003` | V002, multiple initials | all initial declarations |
| `SM004` | V003 | duplicate and original state declarations |
| `SM005` | V004 | duplicate and original transition declarations |
| `SM006` | V005, unknown source | transition and unknown source ID |
| `SM007` | V005, unknown destination | transition and unknown destination ID |
| `SM008` | V006, duplicate `Direct` trigger | conflicting transition declarations |
| `SM009` | V008 | state with no incoming transition |
| `SM010` | V009 | active state with no outgoing transition |
| `SM011` | V010 | terminal state and illegal outgoing transition |
| `SM012` | V011, illegal exit | absorbing state and leaving transition |
| `SM013` | V011, no self-transition | absorbing state declaration |
| `SM014` | V007 | unreachable state and reachable-set summary |
| `SM015` | V012, missing route | source state and missing route |
| `SM016` | V006/V012, duplicate route trigger | conflicting route handlers |
| `SM017` | V012, undeclared route | transition and undeclared route |
| `SM018` | V002, unknown initial state | initial declaration and unknown state ID |
| `SM101` | P001, no terminal | machine definition |
| `SM102` | P001, no terminal path | state and closed-component witness |
| `SM103` | P002 | one concrete cycle witness |
| `SM104` | P003 | one unacknowledged cycle witness |
| `SM105` | incompatible policies | conflicting policy declarations |

Diagnostic codes are part of the conformance contract. Core message wording should be
stable enough for users, but tests should primarily assert codes, locations, and graph
witnesses rather than entire prose strings.

### Diagnostic examples

An unreachable state should produce a focused message:

```text
error[SM014]: state `Archived` is unreachable from initial state `Draft`
```

A strict terminal transition should point at both declarations:

```text
error[SM011]: terminal state `Approved` cannot have outgoing transition `Reopen`
  primary: transition `Reopen`
  related: `Approved` declared terminal here
```

A liveness-policy failure should include a useful witness:

```text
error[SM102]: state `Waiting` cannot reach a terminal state
  closed component: Waiting -> Retrying -> Waiting
```

## Static and dynamic conformance

### Static definitions

A static definition must be rejected during macro expansion or an equivalent
compile-time generation step when any error diagnostic exists. Generated execution APIs
must only be emitted for valid definitions.

This is “compiler-verified” in the practical Rust sense: the definition mechanism runs
complete graph validation while the crate is compiled and emits compiler diagnostics for
invalid input.

### Dynamic definitions

A runtime graph builder must expose validation before execution:

```text
validate(definition) -> Result<ValidatedMachine, ValidationReport>
```

Execution must require a `ValidatedMachine` or an equivalent proof token. A raw dynamic
definition must not be executable through the canonical API.

### Parity requirement

Given equivalent graph definitions and policies, static and dynamic validation must
produce the same set of diagnostic codes and graph witnesses. Source presentation may
differ.

The preferred implementation direction is a shared, effect-free validation core over
the abstract graph. Macro parsing and runtime builders should both lower into that core.

## Conformance examples

The notation below is illustrative and not a proposed public DSL.

### Valid: single terminal state

```text
initial terminal Done
```

This satisfies V001–V012. It also satisfies `must_reach_terminal`.

### Valid: review loop with terminal completion

```text
initial active Draft
active Review
terminal Approved

Draft  -- Submit         --> Review
Review -- RequestChanges --> Draft
Review -- Approve        --> Approved
```

This is valid by default and under `must_reach_terminal`. It is invalid under `acyclic`
unless the review loop is removed.

### Valid: persistent service

```text
initial active Running
Running -- Tick --> Running
policy persistent
```

The self-transition satisfies V009. No terminal state is required.

### Valid: absorbing cancellation

```text
initial active Running
absorbing Cancelled

Running   -- Cancel  --> Cancelled
Cancelled -- Observe --> Cancelled
```

This is valid without `must_reach_terminal`. It is invalid under `acyclic` and
`must_reach_terminal`.

### Invalid: disconnected cycle

```text
initial active Start
terminal Done
active A
active B

Start -- Finish --> Done
A     -- Next   --> B
B     -- Again  --> A
```

`A` and `B` have incoming and outgoing edges but fail V007. Both receive `SM014`.

### Invalid: active dead end

```text
initial active Start
```

`Start` fails V009 with `SM010`. Declaring it terminal would make the machine valid.

### Invalid: terminal escape

```text
initial terminal Done
terminal Reopened
Done -- Reopen --> Reopened
```

The transition fails V010 with `SM011`. Under the required validation order, the
endpoint-valid edge still participates in reachability analysis, so `Reopened` does not
also receive a cascading `SM014`.

### Invalid: incomplete route coverage

```text
initial active Review
terminal Approved
routes Review = {Approve, RequestChanges}

Review -- Approve --> Approved
```

The missing `RequestChanges` handler fails V012 with `SM015`.

## Required test strategy

### Unit tests for the validation core

Each invariant and policy must have:

- at least one smallest valid graph
- at least one smallest invalid graph
- a test asserting the diagnostic code and primary declaration
- boundary tests for self-transitions and the initial state

### Table-driven graph tests

The initial conformance matrix must include:

| Case | Expected result |
| --- | --- |
| empty machine | `SM001`, then initial analysis suppressed |
| no initial state | `SM002` |
| multiple initial states | `SM003` |
| initial declaration names an unknown state | `SM018` |
| duplicate state ID | `SM004` |
| duplicate transition ID | `SM005` |
| unknown source or destination | `SM006` or `SM007` |
| duplicate direct trigger | `SM008` |
| duplicate routed trigger | `SM016` |
| non-initial state with no incoming edge | `SM009` |
| active state with no outgoing edge | `SM010` |
| terminal state with any outgoing edge | `SM011` |
| absorbing state with an external edge | `SM012` |
| absorbing state without a self-edge | `SM013` |
| disconnected state | `SM014` |
| disconnected cycle | `SM014` for each unreachable state |
| missing route handler | `SM015` |
| undeclared route handler | `SM017` |
| no terminal under `must_reach_terminal` | `SM101` |
| closed active component under `must_reach_terminal` | `SM102` |
| self-loop under `acyclic` | `SM103` |
| multi-state cycle under `acyclic` | `SM103` |
| unacknowledged cycle under `cycles_explicit` | `SM104` |
| contradictory policies | `SM105` |

### Compile-fail tests

Every structural diagnostic supported by the static definition API must have a
compile-fail fixture. Tests must assert the diagnostic code and the source span selected
as primary.

### Static/runtime parity tests

Equivalent static fixtures and runtime graph values must be validated against the same
expected diagnostic-code set. These tests protect the dynamic escape hatch from becoming
a second, weaker definition of validity.

### Property tests

Once the validation core exists, generated finite graphs should verify at least these
properties:

- a successful validation has exactly one initial state
- every state in a successful validation is reachable
- every terminal state has out-degree zero
- every absorbing state's outgoing destinations equal itself
- every active state has positive out-degree
- route coverage is a set equality for every branching state
- adding a disconnected state to a valid graph makes validation fail

## Algorithmic expectations

The required analyses are conventional and should not drive public API complexity:

- identifier and trigger checks: hash maps and sets
- reachability: depth-first or breadth-first traversal from the initial state
- cycle and closed-component analysis: Tarjan's or Kosaraju's strongly connected
  components algorithm
- terminal-path analysis: reverse reachability beginning at terminal states
- route coverage: set comparison per source state

Validation should be linear in the graph size, excluding diagnostic sorting:

```text
O(|States| + |Transitions| + |Routes|)
```

Deterministic diagnostics require stable declaration ordering or an explicit sort after
analysis. Hash-map iteration order must not leak into user-visible output.

## Implementation sequencing

The specification suggests this implementation order:

1. Introduce an internal graph IR with stable declaration locations.
2. Implement the unconditional validator as an effect-free function.
3. Add stable diagnostics and the table-driven conformance suite.
4. Implement optional policy analyses.
5. Prototype a runtime definition builder that produces the IR.
6. Prototype static macro syntax that lowers to the same IR.
7. Add compile-fail and static/runtime parity fixtures.
8. Generate execution-facing types only after validation succeeds.

This order deliberately validates the model before committing to DSL ergonomics.

## Decisions intentionally deferred

The following decisions are not required to begin the validation core:

- exact macro syntax
- whether route sets come from enums, macro declarations, or both
- generated type names and module layout
- how transition effects are registered
- whether guards are part of transition identity
- sync and async executor trait signatures
- persistence and serialization format
- semantic contract integration

Any later decision that changes the abstract graph or validity rules must update this
specification and its conformance tests explicitly.

## Acceptance criteria for the design phase

This specification is ready to guide implementation when:

- every roadmap invariant maps to a normative rule here
- terminal, absorbing, active, cyclic, and persistent semantics are unambiguous
- unconditional validity and optional policy checks are clearly separated
- diagnostic codes cover every normative failure
- the conformance matrix is implementable without choosing a public DSL
- static and dynamic modes share one validity contract
