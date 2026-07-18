# Orichalcum: An Agent Orchestration Framework in Rust

**License**: [MIT](LICENSE) | **Crates.io**: [v0.5.0](https://crates.io/crates/orichalcum) | **Docs**: [docs.rs](https://docs.rs/orichalcum)

A type-safe, composable agent orchestration framework for building complex, multi-step workflows.

## Status

⚠️ **This library is in early development (0.x). The API is unstable and may change.**

## What is Orichalcum?

Orichalcum brings Rust's ownership model and type system to agent orchestration. It is
designed for workflows where explicit state, composable operations, and compile-time
feedback matter as much as runtime flexibility.

Orichalcum is a spiritual successor to Python's [PocketFlow](https://github.com/The-Pocket/PocketFlow), inheriting its philosophy of extreme composability. It allows you to define complex workflows (or "Flows") by chaining together simple, reusable components ("Nodes"). Each Node is a self-contained unit of work that can read from and write to a shared state, making decisions about what Node to execute next.

### Core Concepts

*   **Node**: The fundamental unit of work. A `Node` encapsulates a piece of logic with three steps: `prep` (prepare inputs), `exec` (execute the core logic), and `post` (process results and update state).
*   **Flow**: A special `Node` that orchestrates a graph of other `Node`s. It manages the execution sequence based on the outputs of each `Node`.
*   **Shared State**: A `HashMap` that is passed through the entire `Flow`. Nodes can read from this state to get context and write to it to pass results to subsequent nodes.
*   **Semantic Layer (v0.5.0)**: Define structural contracts for your nodes using `Signature` and validate dynamic workflow data flow before execution.

## Installation

Add Orichalcum to your project's `Cargo.toml`:

```toml
[dependencies]
orichalcum = "0.5.0"

# For LLM features (Ollama, Gemini, DeepSeek)
# orichalcum = { version = "0.5.0", features = ["llm"] }

# For the experimental compiler-verified graph API
# orichalcum = { version = "0.5.0", features = ["experimental-graph"] }
```

For repository development and release verification, see [CONTRIBUTING.md](CONTRIBUTING.md).
The compiler-verified graph direction is specified in [docs/STATE_MACHINE_VALIDITY.md](docs/STATE_MACHINE_VALIDITY.md).
Typed failure ownership and mutation behavior are specified in [docs/TYPED_EXECUTION_SEMANTICS.md](docs/TYPED_EXECUTION_SEMANTICS.md).

### Experimental compiler-verified graphs

The `experimental-graph` feature exposes the current whole-graph procedural macro. It
validates structural invariants during compilation and generates phase-specific execution
methods. This preview API may change between 0.x releases.

```rust
use std::convert::Infallible;
use orichalcum::experimental_state_machine;

experimental_state_machine! {
    machine review_flow;
    initial Draft;
    active Draft;
    terminal Approved;
    transition approve: Draft -> Approved;
}

let approved = review_flow::Definition::start(Vec::<&str>::new())
    .approve(|events| {
        events.push("approved");
        Ok::<(), Infallible>(())
    })
    .expect("the effect is infallible");

assert_eq!(approved.into_data(), ["approved"]);
```

## Quick Start: Semantic LLM Nodes (v0.5.0)

Semantic nodes carry explicit input/output contracts and can be sealed against further
configuration changes before execution.

```rust
use orichalcum::prelude::*;
use orichalcum::{Client, HashMap, signature};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // 1. Initialize an LLM client (requires "llm" feature)
    let client = Client::new().with_ollama();

    // 2. Define a semantic signature
    let signature = signature!("document -> summary, sentiment");

    // 3. Build a semantic node
    let node = client.semantic_node()
        .signature(signature)
        .instruction("Summarize the document and analyze its sentiment.")
        .task_id("doc_processor_v1")
        .seal(); // Returns a SealedNode (wrapped in Executable)

    // 4. Run it in a flow
    let flow = AsyncFlow::new(node);
    let mut state = HashMap::new();
    state.insert("document".to_string(), "Rust is a multi-paradigm, general-purpose programming language...".into());

    flow.run(&mut state).await;

    println!("Summary: {}", state.get("summary").unwrap());
    println!("Sentiment: {}", state.get("sentiment").unwrap());
}
```

## Typed Workflows: Phase-Aware Branching (v0.5.0)

Orichalcum also ships a typed workflow API for phase-aware orchestration. On this path, the framework owns legal branch execution while the caller owns the semantic meaning of each business outcome.

```rust
use orichalcum::typed::{
    BranchBuildError, BranchExecuteError, Flow, FlowState, Next, StateNode, Transition,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct Draft;
#[derive(Debug, Clone, PartialEq, Eq)]
struct Review;
#[derive(Debug, Clone, PartialEq, Eq)]
struct Approved;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ReviewData {
    document: String,
    approved: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReviewDecision {
    Approve,
    RequestChanges,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ReviewError {
    EmptyDocument,
}

#[derive(Debug)]
enum ReviewWorkflowError {
    Review(ReviewError),
    Build(BranchBuildError<ReviewDecision>),
    Execute(BranchExecuteError<ReviewDecision, ReviewError>),
}

impl From<ReviewError> for ReviewWorkflowError {
    fn from(value: ReviewError) -> Self {
        Self::Review(value)
    }
}

impl From<BranchBuildError<ReviewDecision>> for ReviewWorkflowError {
    fn from(value: BranchBuildError<ReviewDecision>) -> Self {
        Self::Build(value)
    }
}

impl From<BranchExecuteError<ReviewDecision, ReviewError>> for ReviewWorkflowError {
    fn from(value: BranchExecuteError<ReviewDecision, ReviewError>) -> Self {
        Self::Execute(value)
    }
}

struct SubmitForReview;

impl Transition<Draft, ReviewData> for SubmitForReview {
    type NextPhase = Review;
    type Error = ReviewError;

    fn advance(&self, state: &mut FlowState<Draft, ReviewData>) -> Result<(), Self::Error> {
        if state.data().document.trim().is_empty() {
            return Err(ReviewError::EmptyDocument);
        }

        Ok(())
    }
}

struct ReviewNode;

impl StateNode<Review, ReviewData> for ReviewNode {
    type Route = ReviewDecision;
    type Error = ReviewError;

    fn run(
        &self,
        state: &mut FlowState<Review, ReviewData>,
    ) -> Result<Next<Self::Route>, Self::Error> {
        state.data_mut().approved = true;
        Ok(Next::Route(ReviewDecision::Approve))
    }
}

struct Approve;

impl Transition<Review, ReviewData> for Approve {
    type NextPhase = Approved;
    type Error = ReviewError;

    fn advance(
        &self,
        _state: &mut FlowState<Review, ReviewData>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

struct RequestChanges;

impl Transition<Review, ReviewData> for RequestChanges {
    type NextPhase = Draft;
    type Error = ReviewError;

    fn advance(
        &self,
        state: &mut FlowState<Review, ReviewData>,
    ) -> Result<(), Self::Error> {
        state.data_mut().approved = false;
        Ok(())
    }
}

enum ReviewOutcome {
    Approved(Flow<Approved, ReviewData>),
    Draft(Flow<Draft, ReviewData>),
    StillInReview(Flow<Review, ReviewData>),
}

let review_flow = Flow::<Draft, _>::new(ReviewData {
    document: "Ship it".into(),
    approved: false,
})
.transition(SubmitForReview)?;

let outcome: ReviewOutcome = review_flow
    .step(&ReviewNode)?
    .branch::<ReviewOutcome>()
    .on(ReviewDecision::Approve, Approve, ReviewOutcome::Approved)?
    .on(
        ReviewDecision::RequestChanges,
        RequestChanges,
        ReviewOutcome::Draft,
    )?
    .on_finish(ReviewOutcome::StillInReview)?
    .finish()?;

match outcome {
    ReviewOutcome::Approved(flow) => assert!(flow.data().approved),
    ReviewOutcome::Draft(_) => unreachable!("example review always approves"),
    ReviewOutcome::StillInReview(_) => unreachable!("review node should route"),
}
# Ok::<(), ReviewWorkflowError>(())
```

The typed API enforces phase legality for nodes, transitions, and registered branch handlers at compile time. It does not yet provide compile-time exhaustive route coverage; missing route handlers and missing finish handlers are surfaced as runtime branch execution errors.

Use `step_recovering`, `transition_recovering`, and `finish_recovering` when a failure
must return the source-phase flow for inspection or retry. Mutations performed before an
error remain visible; Orichalcum does not silently roll domain data back.

## Traditional Example: A Simple Sync Flow

Orichalcum still supports pure Rust logic nodes for local processing.

```rust
use orichalcum::prelude::*;

#[derive(Clone)]
struct AddNameLogic;

impl NodeLogic for AddNameLogic {
    fn post(&self, shared: &mut HashMap<String, NodeValue>, _prep: NodeValue, _exec: NodeValue) -> Option<String> {
        shared.insert("name".to_string(), "Orichalcum".into());
        Some("default".to_string())
    }
    fn clone_box(&self) -> Box<dyn NodeLogic> { Box::new(self.clone()) }
}

#[derive(Clone)]
struct GreetLogic;

impl NodeLogic for GreetLogic {
    fn post(&self, shared: &mut HashMap<String, NodeValue>, _prep: NodeValue, _exec: NodeValue) -> Option<String> {
        if let Some(name) = shared.get("name").and_then(|v| v.as_str()) {
            println!("Hello, {}!", name);
        }
        None
    }
    fn clone_box(&self) -> Box<dyn NodeLogic> { Box::new(self.clone()) }
}

fn main() {
    let start_node = Node::new(AddNameLogic).next(Executable::Sync(Node::new(GreetLogic)));
    let flow = Flow::new(start_node);
    let mut state = HashMap::new();
    flow.run(&mut state);
}
```

## Features

*   **Semantic Layer**: Define I/O contracts with `Signature` for contract-aware data flow.
*   **Telemetry (v0.5.0)**: Built-in tracing for I/O, model names, and execution timestamps.
*   **Unified LLM Builders**: Fluent API for `Gemini`, `DeepSeek`, and `Ollama`.
*   **Async & Parallel**: First-class support for `tokio` and parallel batch processing.
*   **Nix Support**: Includes `flake.nix` for a reproducible development environment.

## Contributing

Contributions are welcome! Please feel free to open an issue or submit a pull request.

## License

This project is licensed under the [MIT License](LICENSE).
