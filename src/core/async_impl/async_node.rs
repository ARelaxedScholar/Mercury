use std::collections::HashMap;

use crate::sync::node::Node;
use crate::sync_impl::AsAny;
use crate::sync_impl::NodeValue;
use crate::sync_impl::node::NodeCore;

use async_trait::async_trait;

/// Async Node
pub struct AsyncNode {
    pub data: NodeCore,
    pub behaviour: Box<dyn AsyncNodeLogic>,
}

impl AsyncNode {
    pub fn new<L: AsyncNodeLogic>(behaviour: L) -> Self {
        AsyncNode {
            data: NodeCore::default(),
            behaviour,
        }
    }

    pub fn set_params(&mut self, params: HashMap<String, NodeValue>) {
        self.data.params = params;
    }
    pub fn next(self, node: Node) -> Self {
        self.next_on(node, "default")
    }
    pub fn next_on(mut self, node: Node, action: &str) -> Self {
        if self.data.successors.contains_key(action) {
            log::warn!(
                "Warning: Action {} was found in successors, Overwriting key {}.",
                &action,
                &action
            );
        }
        self.data.successors.insert(action.to_string(), node);
        self
    }
}

// More or less the same logic as NodeLogic
#[async_trait]
pub trait AsyncNodeLogic: AsAny + Send + Sync + 'static {
    // Required method
    fn clone_box(&self) -> Box<dyn AsyncNodeLogic>;

    async fn prep_async(
        &self,
        _params: &HashMap<String, NodeValue>,
        _shared: &HashMap<String, NodeValue>,
    ) -> NodeValue;
    async fn exec_async(&self, _input: NodeValue) -> NodeValue;
    async fn post_async(
        &self,
        _shared: &mut HashMap<String, NodeValue>,
        _prep_res: NodeValue,
        _exec_res: NodeValue,
    ) -> Option<String>;
}
