// The main modules (async will be added soon)
pub mod sync;

// LLM feature
#[cfg(feature = "llm")]
pub mod llm;

// Re-export commonly used types for convenience
#[cfg(feature = "llm")]
pub use llm::{Client, LLMError};
