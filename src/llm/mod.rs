/// llm modules
pub mod error;
pub mod ollama;

/// LLM client (wrapper around reqwest::Client)
/// which is bound to specific providers (can be many)
/// to get a generation from the provider
#[derive(Clone, Debug)]
pub struct Client<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// reqwest::Client, does the querying
    client: reqwest::Client,
    /// A cons list (actually a tuple) which contains all the providers
    /// bound to this Client. Used to specify which methods can be called.
    state: S,
}

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

/// Type-level proof to enforce which methods can be used.
/// This is how the compiler is able to enforce that Client HasProvider<Ollama> is true,
/// and that the associated methods can be called.
pub trait HasProvider<Provider> {}

/// Where the actual proof is defined
mod private {
    /// Found at the current index of the cons list
    struct Here;
    /// Not found at the current index of the cons list
    struct There<I>(std::marker::PhantomData<I>);
    trait FindProvider<T, Index> {}

    /// Base Case (We found it at current index)
    impl<T, Tail> FindProvider<T, Here> for (T, Tail) {}

    /// Recursive Case (We haven't found it yet)
    impl<Head, Tail, T, Index> FindProvider<T, There<Index>> for (Head, Tail) where 
        Tail : FindProvider<T, Index> {}
}



