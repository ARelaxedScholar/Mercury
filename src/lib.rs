// The main modules
mod core;

// LLM feature
#[cfg(feature = "llm")]
pub mod llm;

// Re-export commonly used types for convenience
#[cfg(feature = "llm")]
pub use llm::{Client, error::LLMError};
