//! # Orichalcum
//!
//! A brutally-safe, composable agent orchestration framework for building complex,
//! multi-step workflows in Rust.
//!
//! ## Features
//!
//! - **Memory-Safe Workflows**: Let the compiler catch errors at compile time
//! - **Sync & Async Support**: Full support for both synchronous and asynchronous execution
//! - **Composable Design**: Build complex workflows by composing simple, reusable nodes
//! - **Optional LLM Integration**: Built-in support for LLM providers (feature-gated)
//! - **Pick-and-choose Philosophy**: I have a few ideas of things I could add, but everything shall always be feature-gated.
//!
//! ## Quick Start
//!
//! ```rust
//! use orichalcum::prelude::*;
//! use std::collections::HashMap;
//!
//! // Define your node logic
//! #[derive(Clone)]
//! struct MyLogic;
//!
//! impl NodeLogic for MyLogic {
//!     fn prep(&self, _params: &HashMap<String, NodeValue>, _shared: &HashMap<String, NodeValue>) -> NodeValue {
//!         NodeValue::Null
//!     }
//!     
//!     fn exec(&self, _input: NodeValue) -> NodeValue {
//!         NodeValue::Null
//!     }
//!     
//!     fn post(&self, shared: &mut HashMap<String, NodeValue>, _prep: NodeValue, _exec: NodeValue) -> Option<String> {
//!         shared.insert("result".to_string(), "done".into());
//!         None
//!     }
//!     
//!     fn clone_box(&self) -> Box<dyn NodeLogic> {
//!         Box::new(self.clone())
//!     }
//! }
//!
//! // Create and run a simple flow
//! let node = Node::new(MyLogic);
//! let mut flow = Flow::new(node);
//! let mut state = HashMap::new();
//! flow.run(&mut state);
//!
//! // Verify the result
//! assert_eq!(state.get("result").unwrap().as_str().unwrap(), "done");
//! ```
//!
//! ## Module Organization
//!
//! - `sync`-style types: Synchronous node and flow implementations
//! - `async_impl`-style types: Asynchronous node and flow implementations
//! - [`prelude`]: Commonly used types and traits (import with `use orichalcum::prelude::*`)
//! - [`sync_prelude`]: Only synchronous types (import with `use orichalcum::sync_prelude::*`)
//! - [`async_prelude`]: Only asynchronous types (import with `use orichalcum::async_prelude::*`)

// ============================================================================
// Core Module
// ============================================================================

mod core;

// ============================================================================
// Public Re-exports - Granular Imports
// ============================================================================

// Core types
pub use core::Executable;
pub use core::semantic::registry::{OptimizationRecord, OptimizationRegistry};
pub use core::semantic::signature::{Field, Signature};
pub use core::semantic::{Promptable, Sealable};
pub use core::telemetry::{MemoryTelemetry, Telemetry, TraceEntry};
pub use core::typed::{
    Branch, BranchBuildError, BranchBuildFailure, BranchBuilder, BranchExecuteError, BranchFailure,
    ConfiguredBranchBuilder, ExecutionFailure, FlowState, Next, NodeFailure, OperationKind,
    StateNode, Transition, TransitionFailure,
};
pub use core::validation::{KeyAvailability, ValidationIssue, ValidationResult};

// Synchronous implementations
pub use core::sync_impl::NodeValue;
pub use core::sync_impl::batch_flow::BatchFlow;
pub use core::sync_impl::batch_node::{BatchLogic, new_batch_node};
pub use core::sync_impl::flow::{Flow, FlowLogic};
pub use core::sync_impl::node::{Node, NodeCore, NodeLogic};

// Asynchronous implementations
pub use core::async_impl::async_batch_node::{AsyncBatchLogic, new_async_batch_node};
pub use core::async_impl::async_flow::{AsyncFlow, AsyncFlowLogic};
pub use core::async_impl::async_node::{AsyncNode, AsyncNodeLogic};
pub use core::async_impl::async_parallel_batch_node::{
    AsyncParallelBatchLogic, new_async_parallel_batch_node,
};

/// Typed workflow API organized under its own module so the existing dynamic `Flow`
/// can remain stable while the typed model matures.
pub mod typed {
    pub use crate::core::typed::{
        Branch, BranchBuildError, BranchBuildFailure, BranchBuilder, BranchExecuteError,
        BranchFailure, ConfiguredBranchBuilder, ExecutionFailure, Flow, FlowState, Next,
        NodeFailure, OperationKind, StateNode, Transition, TransitionFailure,
    };
}
// ============================================================================
// Prelude Modules - Convenient Bulk Imports
// ============================================================================

/// The main prelude: imports everything you need for both sync and async workflows.
///
/// # Example
/// ```rust
/// use orichalcum::prelude::*;
/// ```
pub mod prelude {
    pub use super::{
        AsyncBatchLogic,
        AsyncFlow,
        AsyncFlowLogic,
        // Async
        AsyncNode,
        AsyncNodeLogic,
        AsyncParallelBatchLogic,
        BatchFlow,

        BatchLogic,
        // Core
        Executable,
        Flow,
        FlowLogic,
        KeyAvailability,
        MemoryTelemetry,
        // Sync
        Node,
        NodeCore,
        NodeLogic,
        NodeValue,
        OptimizationRecord,
        OptimizationRegistry,
        Promptable,
        Sealable,
        Telemetry,
        TraceEntry,
        ValidationIssue,
        ValidationResult,
        new_async_batch_node,
        new_async_parallel_batch_node,
        new_batch_node,
    };
}

/// Prelude for synchronous-only workflows.
///
/// Use this when you only need synchronous execution and want to avoid
/// importing async types.
///
/// # Example
/// ```rust
/// use orichalcum::sync_prelude::*;
/// ```
pub mod sync_prelude {
    pub use super::{
        BatchFlow, BatchLogic, Executable, Flow, FlowLogic, Node, NodeCore, NodeLogic, NodeValue,
        new_batch_node,
    };
}

/// Prelude for asynchronous-only workflows.
///
/// Use this when you only need async execution and want to avoid
/// importing sync types.
///
/// # Example
/// ```rust
/// use orichalcum::async_prelude::*;
/// ```
pub mod async_prelude {
    pub use super::{
        AsyncBatchLogic, AsyncFlow, AsyncFlowLogic, AsyncNode, AsyncNodeLogic,
        AsyncParallelBatchLogic, Executable, NodeValue, new_async_batch_node,
        new_async_parallel_batch_node,
    };
}

/// Prelude for typed, phase-aware workflows.
pub mod typed_prelude {
    pub use super::typed::Flow;
    pub use super::{
        Branch, BranchBuildError, BranchBuildFailure, BranchBuilder, BranchExecuteError,
        BranchFailure, ConfiguredBranchBuilder, ExecutionFailure, FlowState, Next, NodeFailure,
        OperationKind, StateNode, Transition, TransitionFailure,
    };
}

// ============================================================================
// LLM Feature
// ============================================================================

#[cfg(feature = "llm")]
pub mod llm;

#[cfg(feature = "llm")]
pub use llm::{
    Client,
    error::LLMError,
    ollama::{Ollama, OllamaResponse},
};

// ============================================================================
// Re-export commonly used external types for convenience
// ============================================================================

pub use serde_json::Value as JsonValue;
pub use std::collections::HashMap;

/// Define and validate a complete state-machine graph during macro expansion.
///
/// This API is experimental and may change between Orichalcum 0.x releases.
#[cfg(feature = "experimental-graph")]
pub use orichalcum_macros::experimental_state_machine;

// ============================================================================
// Library Metadata
// ============================================================================

/// The version of this crate.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The name of this crate.
pub const NAME: &str = env!("CARGO_PKG_NAME");
