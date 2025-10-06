use crate::sync::node::{Node, NodeLogic};
use crate::sync::{NodeValue, flow::Flow};

/// A BatchFlow is a `Node` (so orchestrable) which runs
/// a `Flow` many times with different params.
/// Therefore, a `BatchFlow` must deref the `Node`
/// But it's actual exectution logic should be implemented
/// on a struct which carries the `Flow` to batch.
pub struct BatchFlow(Node);

pub struct BatchFlowLogic {
    flow: Flow,
}

impl NodeLogic for BatchFlowLogic {}

impl BatchFlow {
    pub fn new(flow: Flow) -> Self {
        BatchFlow(Node::new(BatchFlowLogic { flow }))
    }
}
