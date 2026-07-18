# Orichalcum Graph-Definition Architecture

## Status

This document records the result of the first graph-definition prototype and compares
the principal compile-time definition strategies. It chooses an architectural direction
without freezing the final public DSL.

Structural validity remains normative in `STATE_MACHINE_VALIDITY.md`. The implementation
core in `crates/orichalcum-definition` is executable evidence for that model. It is
published for transitive Cargo resolution, but its Rust API is not yet a stable public
integration surface.

## Prototype result

The prototype represents a machine independently of execution effects:

```text
MachineDefinition {
    states: [StateDeclaration],
    transitions: [TransitionDeclaration],
    initials: [InitialDeclaration],
    policies: [PolicyDeclaration],
}
```

It supports:

- active, strict terminal, and absorbing state categories
- direct and finite routed transitions
- exactly one explicitly declared initial state
- stable definition locations and diagnostic codes
- reachability and local degree checks
- terminal and absorbing edge restrictions
- exact route coverage
- `must_reach_terminal`, `acyclic`, `cycles_explicit`, and `persistent` policies
- deterministic reachability, reverse-reachability, and strongly connected component
  analysis
- a `ValidatedMachine` proof boundary before execution

The prototype began workspace-private. The validated execution slice now justifies an
experimental root-crate feature, with its implementation crates published in lockstep so
ordinary crates.io dependency resolution works.

## State-category prototypes

The following notation is illustrative and is not the selected DSL.

### Strict terminal

```text
initial active Running
terminal Completed
Running -- Complete --> Completed
```

`Completed` has no outgoing transition. A transition from it is rejected with `SM011`.

### Absorbing

```text
initial active Running
absorbing Cancelled
Running   -- Cancel  --> Cancelled
Cancelled -- Observe --> Cancelled
```

The self-transition is required. An edge from `Cancelled` to any other state is rejected
with `SM012`.

### Cyclic with terminal possibility

```text
initial active Draft
active Review
terminal Approved
Draft  -- Submit         --> Review
Review -- RequestChanges --> Draft
Review -- Approve        --> Approved
policy must_reach_terminal
```

This is structurally valid and has a possible terminal path from every nonterminal
state. It is invalid under `acyclic`. Under `cycles_explicit`, the two transitions in the
review loop must be acknowledged.

### Persistent

```text
initial active Running
Running -- Tick --> Running
policy persistent
```

This explicitly communicates that terminal reachability is not intended. It remains
subject to all unconditional structural rules.

## Requirements for a static definition front end

Whichever syntax is selected must:

- present the complete graph to one compile-time analysis
- preserve source spans for states, transitions, routes, and policies
- lower into the same span-neutral IR used by runtime definitions
- reject invalid structure before generating execution APIs
- make finite route sets inspectable
- generate ordinary Rust types and documentation users can understand
- avoid encoding graph algorithms in user-visible type signatures

These requirements rule out any approach that merely builds a runtime value and calls
it “compiler verified.”

## Candidate A: function-like procedural macro

Illustrative shape:

```rust,ignore
state_machine! {
    machine ReviewFlow;
    initial Draft;

    active Draft routes { Submit };
    active Review routes { Approve, RequestChanges };
    terminal Approved;

    Submit: Draft -> Review;
    Approve: Review -> Approved;
    RequestChanges: Review -> Draft [cycle];

    policy must_reach_terminal;
}
```

### Strengths

- sees the complete topology in one token tree
- can run arbitrary linear-time graph validation during expansion
- can attach diagnostics to exact source spans
- can generate marker types, definition metadata, and phase-local execution methods
- naturally enforces that generated APIs exist only after validation succeeds
- syntax can stay domain-focused rather than exposing IR construction mechanics

### Costs

- requires a procedural macro crate
- syntax needs dedicated parsing and compile-fail fixtures
- IDE completion inside the macro is weaker than ordinary method-call completion
- shared validation must be factored so macro-time and runtime behavior cannot drift
- generated-code errors must be carefully hidden behind user-facing diagnostics

## Candidate B: ordinary runtime builder

Illustrative shape:

```rust,ignore
Machine::builder()
    .initial::<Draft>()
    .active::<Draft>()
    .terminal::<Approved>()
    .transition::<Submit, Draft, Review>()
    .build()
```

### Strengths

- familiar Rust method syntax and completion
- useful for genuinely runtime-defined graphs
- easy to construct conditionally from configuration
- naturally returns `Result<ValidatedMachine, ValidationReport>`

### Limitations

- an ordinary builder executes at runtime and cannot provide the required compiler
  rejection
- typestate builders can prove local call ordering but cannot ergonomically perform
  arbitrary whole-graph reachability and SCC analysis
- encoding the entire graph in nested generic parameters would expose the type-level
  graph calculus the roadmap explicitly seeks to avoid
- runtime builder errors have values and declaration IDs, not precise Rust source spans

An ordinary builder is the correct dynamic front end, not the canonical static front
end.

## Candidate C: macro-backed builder syntax

This approach places builder-looking tokens inside a procedural macro:

```rust,ignore
state_machine! {
    Machine::new("ReviewFlow")
        .initial::<Draft>()
        .active::<Draft, routes!(Submit)>()
        .terminal::<Approved>()
        .transition::<Submit, Draft, Review>()
}
```

The macro parses the restricted method chain; it is not executing the ordinary Rust
builder.

### Strengths

- visually familiar to builder-oriented Rust users
- still exposes the whole graph to macro-time validation
- can share names and concepts with the runtime builder

### Costs

- looks like arbitrary Rust while actually accepting a restricted private grammar
- parser errors can surprise users when normal expressions or control flow are rejected
- method-chain noise obscures the graph topology
- source spans are available, but route/edge relationships are less visually direct
- risks coupling the long-term syntax to temporary builder method names

This remains a viable alternate front end, but not the preferred initial one.

## Candidate D: distributed derives or attributes

Illustrative shape:

```rust,ignore
#[derive(State)]
#[state(initial, active)]
struct Draft;

#[derive(Transition)]
#[transition(from = Draft, to = Review)]
struct Submit;
```

Separate derive expansions cannot reliably inspect every declaration in the crate as one
graph. A registry generated through linker tricks, inventory crates, or build scripts
would move validation away from focused macro diagnostics and complicate portability.

Attributes may decorate items generated by a central macro, but distributed derives
should not be the source of truth for whole-graph validity.

## Candidate E: const-evaluated builder

A const builder could theoretically assemble arrays and panic during constant evaluation.
It is not preferred because:

- ergonomic string and collection handling is restricted in const contexts
- diagnostics would point into const machinery rather than graph declarations
- graph witnesses and multiple stable diagnostics are difficult to present well
- generated phase-specific APIs still require a macro or extensive manual declarations

Const assertions may supplement generated artifacts, but they are not the primary
validator.

## Decision

The preferred static front end is a compact function-like procedural macro over a
declarative graph syntax.

The preferred dynamic front end is an ordinary runtime builder.

Both lower into one span-neutral definition and validation core. A macro-backed builder
may be explored later as alternate syntax, but the first implementation should optimize
for graph readability and diagnostic quality.

This selects the architecture, not the final tokens. Names, punctuation, nesting, route
declaration syntax, and generated method names remain intentionally deferred until
compile-fail prototypes can compare ergonomics.

## Shared-core packaging

A procedural macro cannot depend on the main runtime crate if the runtime crate also
re-exports that macro. The implementation should therefore evolve toward three roles:

```text
definition core
  - span-neutral graph IR
  - validator and stable diagnostic codes
  - no runtime executor and no proc_macro types

macro front end
  - parser and source-span adapter
  - lowers tokens into definition core
  - turns diagnostics into compiler errors
  - generates static descriptors and typed execution APIs

runtime crate
  - dynamic builder lowers into definition core
  - executes only ValidatedMachine values
  - re-exports the macro front end
```

The macro and definition core are published as lockstep implementation crates so the
root runtime package can resolve and re-export the procedural macro. The definition core
still avoids dependencies on `syn`, `quote`, async runtimes, or effect traits.

The original crate-private prototype has now been extracted into
`crates/orichalcum-definition` for the proc-macro implementation. The crate is published
for dependency resolution and remains span-neutral; its direct Rust API is experimental.

## Diagnostic mapping

The shared validator operates on abstract `DefinitionLocation` values. Front ends keep a
location map:

- the macro maps locations to token spans
- the runtime builder maps locations to declaration ordinals and optional caller labels
- serialized definitions may map locations to source file/line metadata

Stable diagnostic codes, related locations, and graph witnesses come from the shared
core. Only presentation is front-end-specific.

This preserves the static/runtime parity requirement without contaminating the validator
with compiler APIs.

## Generated artifact boundary

After validation, the macro should generate two distinct products:

1. a static, inspectable graph descriptor containing states, transitions, routes, and
   policies
2. typed execution-facing handles and methods that can only name legal local operations

The descriptor is not reconstructed by inspecting generated types. Definition remains
the source; execution APIs are derived output.

Invalid input must stop before either execution product is usable.

## Prototype findings that changed the specification

The executable prototype exposed two details that were underspecified:

- an initial declaration can name an unknown state, now diagnosed as `SM018`
- a state with no incoming edge should receive focused `SM009`, while unreachable states
  with incoming edges receive `SM014`

This is exactly the role of the prototype: harden the implementation-neutral contract
before public syntax makes ambiguities expensive.

## First implementation spike

The `orichalcum-macros` implementation crate now supports:

- named active, terminal, and absorbing states
- one initial declaration
- direct and routed transitions
- unconditional validation and all four graph policies
- generated zero-sized phase marker types and a static descriptor
- a minimal generated typestate execution skeleton
- fallible call-site transition effects with recoverable source-phase ownership
- generated exhaustive route enums, destination-typed outcome enums, and dispatch
- compile-fail fixtures for structural diagnostics, wrong-phase transitions, and attempts
  to leave terminal states

The initial no-op transition skeleton has been replaced by a thin executable slice.
Direct transitions accept fallible effects, and routed states accept one fallible effect
per exhaustively validated route. A returned error owns the mutated source-phase
execution; a success produces only the declared destination-phase execution.

This is not the final executor. Effects are supplied as call-site closures rather than a
reusable validated binding artifact, node effects do not yet produce routes, and async
effects are absent. The slice proves that graph validation, generated phase legality,
route dispatch, and failure ownership can coexist before public syntax is stabilized.

The compile-fail results support the function-like procedural macro recommendation:
graph diagnostics attach to the declarations users need to repair, while phase-local
misuse falls through to familiar Rust method-resolution diagnostics.

The public preview is packaged as three lockstep-versioned crates. `orichalcum` is the
supported user entry point and re-exports the macro behind `experimental-graph`;
`orichalcum-macros` and `orichalcum-definition` are published implementation packages.
This keeps procedural-macro isolation and the shared validator explicit without asking
users to manage those dependencies directly.

## Acceptance criteria

The architectural comparison is settled when:

- the static and dynamic front ends have distinct, honest roles
- both front ends share one validator
- the static approach can inspect the complete graph and emit focused spans
- public graph readability wins over type-level cleverness
- the prototype remains free to evolve before public API stabilization
