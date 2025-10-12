use thiserror::Error;

#[derive(Debug, Error)]
pub enum LLMError {
    #[error("Error occurred during Ollama call: {0}")]
    OllamaError(#[from] reqwest::Error),
}
