/// llm modules
pub mod error;
pub mod ollama;

use ollama::Ollama;
use std::marker::PhantomData;

/// LLM client (wrapper around reqwest::Client)
/// which is bound to specific providers (can be many)
/// to get a generation from the provider
pub struct Client<S> {
    /// reqwest::Client, does the querying
    client: reqwest::Client,
    /// A marker for the current state of the Client (which methods it has a config bound with)
    state: PhantomData<S>,
    /// Ollama config
    ollama_host: Option<String>,
}

/// Type States
pub struct Enabled;
pub struct Disabled;

/// The Marker for the states that are enabled
pub struct Providers<OllamaState> {
    _ollama: PhantomData<OllamaState>,
}

/// The constructors implementations for stuff
impl Client<Providers<Disabled>> {
    pub fn new() -> Self {
        Client {
            client: reqwest::Client::new(),
            state: PhantomData,
            ollama_host: None,
        }
    }
}

impl Default for Client<Providers<Disabled>> {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder functions to bind a given config to the client
/// These functions are only available when the thing hasn't been bound yet
impl Client<Providers<Disabled>> {
    pub fn with_ollama(self, host: impl Into<String>) -> Client<Providers<Enabled>> {
        Client {
            client: self.client,
            state: PhantomData,
            ollama_host: Some(host.into()),
        }
    }
}

/// HasProvider trait to block functions behind having specific providers
pub trait HasProvider<Provider> {}

/// Implementation of what it means for this to be true for Ollama
impl HasProvider<Ollama> for Providers<Enabled> {}

/// LLM Client is fundamentally a reqwest::Client, but bound if user
/// wants to query some page using the client directly, I think they should be able to.
impl<S: Clone + Send + Sync + 'static> std::ops::Deref for Client<S> {
    type Target = reqwest::Client;
    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

impl<S: Clone + Send + Sync + 'static> std::ops::DerefMut for Client<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.client
    }
}
