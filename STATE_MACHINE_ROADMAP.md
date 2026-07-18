# Orichalcum State Machine Roadmap

## Purpose

This document records the shared intent behind Orichalcum's evolution into a best-in-class Rust framework for building type-safe state machines and agent workflows.

It serves two audiences at once:
- contributors and users who need a clear product-direction narrative
- maintainers who need an engineering roadmap grounded in the current codebase

It is intentionally explicit about what is true today, what is still aspirational, and what tradeoffs are currently deliberate rather than accidental.

---

## Executive summary

Orichalcum is moving from a flexible node/flow orchestration library toward a framework that can express workflows as phase-aware state machines with increasingly strong compile-time guarantees.

The long-term aim is not just “workflow orchestration in Rust.” For workflows whose
structure is declared statically, the aim is that an invalid state-machine graph cannot
be defined successfully. The long-term aim is:
- explicit state transitions
- compiler-enforced phase legality
- compiler-verified graph structure at definition time
- exactly one declared initial state
- reachability and transition-shape guarantees for every declared state
- explicit terminal, absorbing, and cyclic semantics
- exhaustive route-to-transition coverage
- branch handling owned by the framework rather than handwritten closures
- domain-level outcome modeling instead of generic control-flow plumbing
- honest separation between what is type-checked and what is runtime-validated
- a path from today's typed local guarantees toward a more complete typed graph model

As of `0.5.0`, Orichalcum has taken an important first real step:
- typed phases exist
- typed nodes exist
- typed transitions exist
- typed branch registration exists on the canonical builder path
- typed branch outcomes are caller-defined
- route coverage is still runtime-validated
- the older dynamic workflow API still coexists unchanged

That means Orichalcum is now a partially typed workflow framework with a credible state-machine core, but it is not yet the “ultimate type-safe state machine building framework.”

This document explains what that destination means and how to get there without lying about current guarantees.

---

## Product vision

### What Orichalcum should become

Orichalcum should become the Rust framework people reach for when they want to model real workflows as state machines instead of loosely connected callbacks.

The ideal user experience is:
- define phases as ordinary Rust types
- declare the complete static graph in one inspectable definition
- define data once and carry it through the workflow honestly
- define legal nodes and transitions in the phases where they are actually valid
- describe branching in domain terms, not framework terms
- let the compiler reject structurally invalid graphs and impossible paths
- let runtime errors represent real runtime facts, not type-system gaps disguised as panics or stringly typed failures

In that world, a workflow author should feel that Orichalcum is doing three jobs well:
1. preserving the domain model
2. enforcing legal execution structure
3. staying out of the way when the model must remain dynamic

### What “ultimate type-safe state machine framework” means here

It does **not** mean every workflow in the ecosystem must be fully encoded in the type system.

It **does** mean Orichalcum should provide a ladder of precision:
- dynamic orchestration when the workflow is truly runtime-defined
- typed phase safety when the workflow shape is known locally
- typed branch execution when routing is known but coverage remains runtime-owned
- typed graph/state-machine definitions whose general structure is compiler-verified

The framework wins if it lets users choose the strongest truthful model their workflow can support.

### Product principles

1. One concept, one representation.
   - A business state should have one primary representation.
   - A legal transition should be a transition type, not a convention hidden in closures.

2. Domain outcomes are not errors.
   - Approval vs rejection vs retry is not `Result` misuse.
   - Branch outcomes belong in caller-defined enums.

3. Type safety must be truthful.
   - Never claim compile-time coverage where the implementation is doing runtime matching.
   - Better a narrower honest guarantee than a broader false one.

4. Dynamic remains an explicit escape hatch.
   - Some workflows are runtime-defined by nature.
   - Dynamic definitions should receive strong runtime validation and explicit execution errors.
   - Typed APIs should not force those users into awkward pseudo-static designs.
   - The future statically verified engine should not secretly depend on string-routed dynamic execution.

5. The compiler should reject structural mistakes before execution.
   - wrong-phase node execution
   - wrong-phase transitions
   - illegal branch handlers
   - missing or ambiguous initial states
   - unreachable states
   - nonterminal states without outgoing transitions
   - illegal terminal-state transitions
   - unhandled or multiply handled routes

---

## Where we are right now

### Release status

Current release target: `0.5.0`

Observed in the codebase:
- typed workflow builder path released and documented
- README updated for `0.5.0`
- changelog updated for `0.5.0`
- the `0.5.0` release snapshot records passing tests, doctests, `llm` feature tests, and typed example checks in `nix develop`
- the current foundation pass reconciles `Cargo.lock` and `Cargo.nix` and provides a repeatable formatting, test, doctest, Clippy, packaging, and Nix verification gate

### What is already true today

The typed workflow API currently provides:
- `FlowState<P, D>` for phase-tagged state
- `StateNode<P, D>` for phase-local node execution
- `Transition<P, D>` for legal phase-local transitions
- `Next<R>` for typed routing decisions
- `Branch<P, D, R>` as a typed branch result carrier
- canonical branch builder path:
  - `.branch::<O>()`
  - `.on(...)`
  - `.on_finish(...)`
  - `.finish()`
- explicit typed errors:
  - `BranchBuildError<R>`
  - `BranchExecuteError<R, E>`

### What compile-time guarantees exist today

The current typed API enforces:
- nodes may only run in the phase they are defined for
- transitions may only execute from the phase they are defined for
- branch handlers registered through `.on(...)` must use transitions legal from the current phase
- branch route values must match the branch route type
- next-phase typing is preserved across transitions
- one branch builder uses a single transition error type `E`

### What remains runtime-validated today

The current typed API still checks these at runtime:
- whether a produced route has a registered branch handler
- whether `Next::Finish` has a registered finish handler
- whether a duplicate route was registered
- transition execution failure propagation

### What is explicitly **not** true yet

Orichalcum does **not** yet provide:
- compile-time exhaustive branch coverage for arbitrary route enums
- a whole-graph typed state-machine DSL
- proof that every reachable route is handled
- typed async branching equivalent to the sync typed path
- a full replacement for the existing dynamic `Flow` / `NodeLogic` model

### Honest assessment

Right now Orichalcum is:
- an established but still rough dynamic workflow/orchestration implementation
- a promising typed phase-and-transition framework
- an early typed state-machine system
- not yet a fully graph-typed workflow compiler

That is good progress, but only if we continue to document it honestly.

---

## Architectural intent

### The current typed model is phase-first, not graph-first

This is the key design truth.

The typed API today models:
- the phase you are in now
- the node you can run now
- the transitions you may legally apply now
- the branch handlers you may legally attach now

It does **not** yet model the entire workflow graph as a type-level artifact.

That was the right decision for `0.5.0` because it shipped meaningful safety without exploding the public API into an unusable type maze.

### Why the branch builder was the right next step

Before `0.5.0`, branch resolution on the typed path still relied on freeform `resolve(...)` closures.
That meant the framework knew a typed route had been produced, but it still delegated the route-to-transition wiring to ad hoc user code.

The builder changed that.

Now the normal path is framework-owned:
- the caller registers route handlers
- the framework executes the selected legal transition
- the caller receives a domain outcome enum

This is a major design improvement because it moves structural workflow logic out of handwritten closures and into a constrained, composable API.

### Why caller-defined outcome enums matter

This was an important design choice and should remain central.

Branching is not primarily a question of framework control flow. It is a question of domain meaning.

A typed workflow should return outcomes like:
- `Approved(Flow<Approved, D>)`
- `Rejected(Flow<Rejected, D>)`
- `NeedsRevision(Flow<Draft, D>)`

That is better than:
- nested `Either`
- positional branch result types
- overloading `Result` to encode normal business branches

This design should continue to guide future typed APIs.

### Why `resolve(...)` still exists

`resolve(...)` remains public as an advanced escape hatch.
That is acceptable today because:
- it preserves flexibility
- it avoids breaking advanced callers
- it lets the typed API grow without prematurely blocking unusual workflows

But its presence also means we must be careful with claims.
The framework owns branch resolution on the builder path, not as an unconditional global invariant.

---

## Definition-time validity contract

The strategic target is a complete static workflow definition that the framework can
analyze before it generates or exposes an execution interface. Compiler-verified
validity refers first to this general graph structure. It does not claim to prove
arbitrary business logic inside nodes or transitions.

### Structural invariants

Every statically defined state machine should satisfy these rules:

1. Exactly one initial state is declared.
2. Every transition names declared source and destination states.
3. Every declared state is reachable from the initial state.
4. Every non-initial state has at least one incoming transition.
5. Every nonterminal state has at least one outgoing transition.
6. A strict terminal state has no outgoing transitions.
7. An absorbing state may transition to itself but may not transition to another state.
8. Every declared route maps to exactly one legal transition.
9. Duplicate or ambiguous state, route, and transition definitions are rejected.

Reachability is deliberately stronger than merely requiring an incoming edge. A
disconnected cycle has incoming edges but is still invalid because execution can never
reach it from the initial state.

### Structural validity versus liveness policy

Some useful state machines intentionally run forever or contain retry loops. Therefore,
the base validity contract should not claim that every workflow must terminate.

Stronger graph properties should be expressed as explicit, compiler-checked policies,
such as:

- `must_reach_terminal`: every reachable nonterminal state can reach a terminal state
- `acyclic`: no directed cycle is permitted
- `cycles_explicit`: every cycle must be declared intentionally
- `persistent`: termination is not required

The exact public syntax remains a design question. The important distinction is that
structural invariants are unconditional, while liveness constraints are selected
according to the workflow's domain.

### Definition and execution are separate concerns

The complete graph belongs to a definition-time API. Current state, workflow data, and
effects belong to an execution-time API.

This separation should allow one verified graph definition to support synchronous and
asynchronous executors without duplicating the structural model. It should also allow a
runtime-defined graph to reuse the same conceptual schema while replacing compile-time
proofs with explicit validation results.

### Likely implementation direction

Ordinary traits are effective for proving local phase legality, but they cannot easily
inspect every variant and edge in a separately assembled graph. The likely pragmatic
destination is a declarative macro or macro-backed definition builder that can:

- inspect the complete graph
- run reachability and strongly connected component analysis during expansion
- verify route coverage and terminal rules
- emit focused compiler diagnostics
- generate strongly typed execution interfaces

This direction is preferred over a generic type-level graph calculus whose public types
would be difficult to understand or maintain.

---

## Strategic destination

The destination is a layered framework with three coherent modes.

### Mode 1: Dynamic orchestration

This is the existing `NodeLogic` / `Flow` world.
Use it when:
- workflow structure is runtime-defined
- edges are loaded from config or external systems
- callers need maximum flexibility

Goal:
- preserve it where runtime-defined topology is genuinely required
- validate definitions before execution wherever possible
- return explicit runtime errors for facts the compiler cannot prove
- keep it separate from the implementation contract of the statically verified engine

### Mode 2: Typed phase-aware workflows

This is the current `typed` API world.
Use it when:
- phases are known at compile time
- data model is stable enough to carry through typed transitions
- callers want strong local correctness guarantees without graph-level complexity

Goal:
- keep it as the current local-safety path while the complete definition model matures
- avoid over-investing in conveniences that a graph-definition API will supersede
- use it to settle the semantics the generated execution API will need

### Mode 3: Typed graph/state-machine composition

This is the long-term strategic prize and intended canonical mode for statically known workflows.
Use it when:
- workflow structure is known at compile time
- legal states and legal transitions should be encoded centrally
- teams want the compiler to reject impossible workflow wiring before runtime

Goal:
- represent the complete workflow topology in one definition-time artifact
- enforce the structural validity contract during compilation
- make graph legality a first-class modeled concept
- generate execution APIs for the verified definition
- preserve domain outcomes instead of erasing them into generic control machinery

---

## Roadmap

## Phase A — Stabilize the `0.5.x` typed foundation

### Goal
Turn the current typed workflow API into a trustworthy base layer.

### Status
Partially complete.

### Already done
- framework-owned branch builder path shipped
- typed branch docs corrected to distinguish compile-time vs runtime guarantees
- compile-fail proofs cleaned up
- transition failure propagation tested

### Next work
1. Keep the docs strict and honest.
   - No more overclaiming compile-time branch coverage.
   - README, rustdoc, and examples must say the same thing.

2. Restore reproducible release verification.
   - keep `Cargo.lock` and `Cargo.nix` synchronized
   - make formatting, tests, doctests, feature builds, and examples routine release gates
   - ensure leading README examples compile against the documented feature set

3. Settle foundational execution semantics.
   - define what happens to workflow state when a node or transition fails
   - replace panic-, log-, and implicit-default failure paths with explicit errors on the typed path
   - distinguish domain outcomes from execution failures consistently

4. Expand typed examples beyond the review-flow toy case.
   - retry loops
   - explicit terminal states
   - multi-branch workflows
   - workflows that intentionally use `on_finish`

5. Improve typed API discoverability.
   - stronger `typed_prelude` guidance
   - dedicated module-level docs for typed usage patterns
   - migration notes from `resolve(...)` to builder path

6. Avoid premature investment in temporary mechanics.
   - keep the current branch builder correct and documented
   - defer extensive builder sugar and boxed-executor optimization until the graph-definition model is known
   - measure current mechanics only when evidence shows they affect real workflows

### Success criteria
- users can discover the typed path quickly
- docs tell the truth consistently
- failures have explicit, documented semantics
- the release gate is reproducible from a clean checkout

---

## Phase B — Formalize the canonical state-machine model

### Goal
Define the concepts and execution contract that both the handwritten typed API and a
future generated graph API will share.

### Required design work

1. Publish the structural validity specification.
   - define initial-state, reachability, incoming-edge, and outgoing-edge rules precisely
   - define route coverage and duplicate-definition rules
   - distinguish unconditional structural invariants from selectable liveness policies

2. Settle state vocabulary.
   - active state
   - strict terminal state
   - absorbing state
   - cyclic or persistent workflow
   - domain outcome versus execution termination

3. Separate graph definition from execution instances.
   - a definition owns states, transitions, routes, and policies
   - an execution instance owns current state and workflow data
   - generated execution types must not be the only representation of the graph schema

4. Design sync and async as execution adapters over one structure.
   - do not create independent sync and async graph models
   - determine how node and transition effects are represented without weakening structural proofs
   - postpone a second branch-builder hierarchy unless it directly supports the canonical model

### Risks
- freezing the current branch builder into the long-term graph API too early
- confusing execution completion with terminal-state structure
- allowing sync and async APIs to duplicate or contradict the graph model

### Success criteria
- the validity rules are precise enough to test and implement
- terminal, absorbing, and cyclic behavior have unambiguous meanings
- one graph definition can plausibly support both sync and async execution

---

## Phase C — Introduce first-class state categories and definition artifacts

### Status
Implemented as a published implementation-level definition/validation core; the public
runtime-definition API remains pending.

### Goal
Make state categories and the complete workflow definition explicit before attempting
whole-graph compiler verification.

### Why this matters
A serious state-machine framework needs a clear structural story for:
- terminal success
- terminal failure
- terminal cancellation
- absorbing states that may self-transition without leaving
- cyclic or persistent execution
- early execution stop without confusing it with graph terminality

Today, `Next::Finish` plus `.on_finish(...)` is an execution mechanism. It is not a
definition-time declaration that a phase is structurally terminal.

### Required capabilities
1. Explicit state categories.
   - strict terminal states have no outgoing edges
   - absorbing states may only transition to themselves
   - active states require outgoing transitions

2. A definition-time artifact.
   - declares the initial state
   - registers every state, route, and transition centrally
   - carries selected graph policies
   - can be inspected independently of a running workflow

3. A separate execution instance.
   - holds current state and domain data
   - can stop, fail, or produce a domain outcome without changing the graph's structural declarations

4. A runtime validation counterpart.
   - dynamic graph definitions should be checked against the same structural vocabulary
   - failures should be returned as detailed validation errors before execution begins

### Caution
Do not force one product philosophy onto all workflows.
Many workflows have domain-specific notions of completion that should remain caller-defined.
Do not encode “success,” “failure,” or “cancellation” as universal framework meanings
when the domain should own those outcomes.

### Success criteria
- the graph can distinguish active, terminal, and absorbing states structurally
- definition and execution have separate public responsibilities
- runtime-defined graphs can report violations using the same rule set

---

## Phase D — Make static graph validity compiler-verified

### Status
The procedural-macro preview validates the full structural vocabulary during expansion
and generates phase-legal fallible transitions, exhaustive typed route dispatch, and
recoverable source-phase failures. It is re-exported by the root crate behind
`experimental-graph`; DSL and reusable effect-binding stabilization remain pending.

### Goal
Reject structurally invalid static workflow definitions during compilation and generate
strongly typed execution interfaces only for valid graphs.

### This is the major strategic leap
Today, the framework knows whether a transition is legal from the current phase.
This phase makes the complete graph visible at definition time and verifies the agreed
structural invariants before execution code can use it.

### Problems this phase tries to solve
- branch coverage known only at runtime
- topology spread across many local definitions
- no central typed representation of legal state-machine structure
- no compiler proof of a unique initial state
- no compiler proof of reachability, terminality, or route exhaustiveness
- no policy-aware diagnosis of closed nonterminal components or unintended cycles

### Implementation direction

1. Prefer a declarative macro or macro-backed definition builder.
   - the complete graph must be available to one definition-time analysis
   - the syntax should name states and transitions centrally
   - generated Rust types should preserve ergonomic execution without hiding the graph

2. Run unconditional structural checks.
   - exactly one initial state
   - all referenced states declared
   - all declared states reachable from the initial state
   - all non-initial states have incoming transitions
   - all active states have outgoing transitions
   - terminal and absorbing edge restrictions
   - exact route-to-transition coverage

3. Run selected policy checks.
   - terminal reachability
   - acyclicity
   - explicit-cycle requirements
   - persistent workflow allowances

4. Generate focused diagnostics.
   - name unreachable states
   - identify illegal terminal edges
   - identify missing or duplicate route mappings
   - identify nonterminal strongly connected components that violate selected policies

5. Generate execution APIs from the verified definition.
   - preserve phase-local legality
   - preserve caller-owned domain outcomes
   - support sync and async executors over the same structural artifact

### Recommendation
Do not jump straight to a generic type-level graph calculus.
That path can easily become impossible to use.

The graph definition should become the canonical path when topology is statically known.
The existing phase-local API may remain useful as a lower-level building block or
compatibility layer, but static users should not need to opt into weaker runtime route
coverage once the verified definition path is stable.

### Success criteria
- every static definition satisfies the unconditional structural validity contract
- selected liveness policies are checked during compilation
- workflow topology is central, explicit, and inspectable
- exhaustive route wiring no longer depends on runtime registration checks
- public ergonomics remain tolerable for real users

---

## Phase E — Unify typed workflows with semantic validation

### Goal
Bring the existing semantic layer and typed state-machine layer closer together.

### Why this matters
Orichalcum already has:
- semantic contracts
- validation primitives
- sealed nodes
- telemetry

The typed state-machine path should not evolve in isolation.

### Opportunities
1. Typed states with semantic I/O contracts.
   - phase-local transitions can also declare semantic expectations

2. Validation at workflow-definition time.
   - semantic mismatches surfaced before execution
   - graph validation informed by both topology and data contracts

3. Sealed typed workflows.
   - identifiable, validated typed state-machine artifacts

4. Telemetry that understands phases and transitions structurally.
   - trace by phase
   - trace by transition kind
   - trace by domain outcome

5. Shared static and dynamic graph schemas.
   - static definitions receive compiler verification and generated execution types
   - runtime definitions receive explicit validation reports before execution
   - both modes use the same state, transition, terminality, and policy vocabulary

### Success criteria
- typed structure and semantic contracts reinforce each other
- Orichalcum becomes more than “typestate plus some unrelated validation utilities”

---

## Phase F — Publish a crisp public positioning

### Goal
Make the external story match the actual architecture.

### The message should be
Orichalcum is:
- a Rust orchestration framework
- with a canonical compiler-verified mode for statically defined state machines
- with an explicit runtime-validated dynamic mode where topology is not known at compile time
- already strong on local phase legality
- growing toward whole-graph structural guarantees

### The message should **not** be
- “everything is compile-time guaranteed already”
- “the graph is fully typed today”
- “runtime validation is gone”
- “compiler verification proves arbitrary business logic or universal termination”

### Deliverables
- README restructuring
- crate docs restructuring
- examples grouped by dynamic vs typed vs semantic paths
- release notes that track the typed roadmap explicitly

---

## Design rules for future work

These rules should constrain future implementation decisions.

### 1. Never oversell compile-time guarantees
If route coverage is runtime-validated, say so.
If a graph is not type-checked globally, say so.

### 2. Keep domain meaning in user-owned types
The framework should orchestrate; callers should name the business outcomes.

### 3. Prefer additive ladders over forced migrations
Dynamic users should not be punished for not fitting the typed path.
Typed users should not be forced into graph-level machinery before they need it.
When topology is statically known, the verified graph path should ultimately be the
canonical recommendation rather than merely another optional convenience.

### 4. Build from legal transitions outward
Graph models should be composed from truthful local legality, not bolted on with a parallel abstraction that ignores the current typed core.

### 5. Keep definition separate from execution
Graph structure, state categories, and policies belong to the definition.
Current state, data, effects, failures, and domain outcomes belong to execution.

### 6. Use one structural model for sync and async
Execution effects may differ, but graph validity must not depend on whether an executor
is synchronous or asynchronous.

### 7. Avoid type cleverness that destroys maintainability
The goal is stronger guarantees with understandable APIs, not type-system theater.

### 8. Measure performance before redesigning for it
The current boxed-executor branch builder may be good enough for a long time.
Optimize it only when evidence says it matters, especially because generated graph
execution may eventually use a different representation.

---

## Concrete next steps

### Immediate next steps after `0.5.0`
1. **Completed:** Restore a clean, reproducible build and documentation verification gate.
2. **Completed:** Write a focused design note that turns the validity contract in this roadmap into testable rules and diagnostics. See `docs/STATE_MACHINE_VALIDITY.md`.
3. **Completed:** Settle failure ownership and state-mutation semantics for typed nodes and transitions. See `docs/TYPED_EXECUTION_SEMANTICS.md`.
4. **Completed:** Prototype strict terminal, absorbing, cyclic, and persistent definitions without committing to a final DSL. See the implementation-level `crates/orichalcum-definition` validation core and `docs/GRAPH_DEFINITION_ARCHITECTURE.md`.
5. **Completed:** Compare a declarative macro with a macro-backed builder for whole-graph inspection and generated APIs. See `docs/GRAPH_DEFINITION_ARCHITECTURE.md`.
6. **Completed:** Design sync and async execution against the same proposed graph artifact. See `docs/EXECUTION_ADAPTER_ARCHITECTURE.md`.
7. **Completed:** Add realistic typed examples that exercise retry cycles, terminal states, absorbing states, and multi-route coverage. See `examples/typed_lifecycle_flow.rs`.

### Recommended milestone framing
- `0.5.x`: stabilize execution semantics, documentation, and release reproducibility
- `0.6.x`: formalize the graph contract, state categories, policies, and definition/execution boundary
- `0.7.x`: introduce compiler-verified static graph definitions and generated execution interfaces
- later: deepen semantic contracts, telemetry, optimization, and runtime-defined graph validation

[inference] Exact versioning may change. The sequencing matters more than the numbers:
settle semantics first, formalize the graph second, generate compiler-verified execution
only after both are stable.

---

## Current state snapshot

### We have
- dynamic sync/async orchestration
- semantic nodes and validation primitives
- typed phase-aware workflow primitives
- framework-owned branch builder on the typed path
- caller-defined semantic branch outcomes
- explicit branch build/execution errors
- honest docs about runtime vs compile-time boundaries

### We do not yet have
- graph-wide compile-time legality
- compile-time exhaustive branch coverage
- async typed branch parity
- first-class terminal or absorbing state categories beyond current finish handling
- an integrated schema/graph/state-machine definition layer
- compiler-checked reachability or liveness policies

### Therefore
Orichalcum is now a credible typed workflow framework with a clear route toward becoming a stronger state-machine framework, but it has not yet completed that journey.

That is where we are right now.

---

## Final intent

The goal is not to create the most clever Rust types possible.
The goal is to create the most trustworthy workflow framework possible.

Trustworthiness here means:
- the API reflects real workflow structure
- the compiler rejects illegal local moves
- statically declared graphs eventually cannot compile unless they satisfy the structural validity contract
- liveness guarantees are explicit policies rather than implied promises
- runtime errors represent actual runtime facts
- docs say exactly what the implementation earns
- stronger guarantees arrive in deliberate layers rather than hype-driven leaps

If Orichalcum keeps that discipline, it can become not just a Rust workflow library, but a genuinely excellent framework for building explicit, type-safe state machines.
