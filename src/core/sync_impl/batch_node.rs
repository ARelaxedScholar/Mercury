use crate::core::sync_impl::NodeValue;
use crate::core::sync_impl::node::{Node, NodeLogic};
use std::collections::HashMap;

/// ------- BatchNode -------------------------------------------------------------
/// This logic is fairly easy to implement since the core logic is really just about taking
/// many items and applying the logic on all of them. But the more powerful approach here
/// is just have BatchNode be generic over NodeLogic, this way it is composable with `Node`
#[derive(Clone)]
pub struct BatchLogic<L: NodeLogic> {
    logic: L,
}

/// Convenience functions to create new BatchLogic (note that in our approach)
/// A `BatchLogic` and a `BatchNode` are not the same.
/// `BatchLogic` is simply a conceptual struct which marks what we'd want to be batched.
/// `BatchNode` which we define through the composition of a `Node` with a `NodeLogic` which is
/// `Clone`-able, is simply a `Node` which applies its logic to a bunch of items (sequentially.)
impl<L: NodeLogic> BatchLogic<L> {
    pub fn new(logic: L) -> Self {
        BatchLogic { logic }
    }
}

/// The advent of the BatchNode
/// Defining the logic for what is a `BatchLogic` which is a "true" `NodeLogic`.
impl<L: NodeLogic + Clone> NodeLogic for BatchLogic<L> {
    fn prep(
        &self,
        params: &HashMap<String, NodeValue>,
        shared: &HashMap<String, NodeValue>,
    ) -> NodeValue {
        self.logic.prep(params, shared)
    }

    fn exec(&self, items: NodeValue) -> NodeValue {
        // Check that input is indeed an array
        if let Some(arr) = items.as_array() {
            let results: Vec<NodeValue> = arr
                .iter()
                .map(|item| self.logic.exec(item.clone()))
                .collect();

            results.into()
        } else {
            log::error!("items is not an array");
            NodeValue::Null
        }
    }

    fn post(
        &self,
        shared: &mut HashMap<String, NodeValue>,
        prep_res: NodeValue,
        exec_res: NodeValue,
    ) -> Option<String> {
        self.logic.post(shared, prep_res, exec_res)
    }

    fn clone_box(&self) -> Box<dyn NodeLogic> {
        Box::new((*self).clone())
    }
}

/// The `BatchNode` factory
pub fn new_batch_node<L: NodeLogic + Clone>(logic: L) -> Node {
    Node::new(BatchLogic { logic })
}
