pub mod node;
pub mod batch_node;
pub mod flow;

/// The Alias for serde_json::Value since I use it a lot
pub type NodeValue = serde_json::Value;

use std::any::Any;

/// A helper trait that just provides the `as_any` method.
/// Needed for convenient downcasting of `FlowLogic` among other things
/// (More here for separation of concerns, since I could have added that to `NodeLogic` directly
pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}


