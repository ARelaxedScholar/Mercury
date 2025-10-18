use crate::core::async_impl::async_node::{AsyncNode, AsyncNodeLogic};
use crate::core::sync_impl::NodeValue;
use std::collections::HashMap;

#[derive(Clone)]
pub struct AsyncBatchLogic<L: AsyncNodeLogic> {
    logic: L,
}

impl<L: AsyncNodeLogic> AsyncBatchLogic<L> {
    pub fn new(logic: L) -> Self {
        AsyncBatchLogic { logic }
    }
}

impl<L: AsyncNodeLogic + Clone> AsyncNodeLogic for AsyncBatchLogic<L> {
    async fn prep(
        &self,
        params: &HashMap<String, NodeValue>,
        shared: &HashMap<String, NodeValue>,
    ) -> NodeValue {
        self.logic.prep(params, shared).await
    }

    async fn exec(&self, items: NodeValue) -> NodeValue {
        // Check that input is indeed an array
        if let Some(arr) = items.as_array() {
            let results: Vec<NodeValue> = arr
                .iter()
                .map(|item| async {self.logic.exec(item.clone()).await})
                .collect();

            results.into()
        } else {
            log::error!("items is not an array");
            NodeValue::Null
        }
    }

    async fn post(
        &self,
        shared: &mut HashMap<String, NodeValue>,
        prep_res: NodeValue,
        exec_res: NodeValue,
    ) -> Option<String> {
        self.logic.post(shared, prep_res, exec_res).await
    }

    fn clone_box(&self) -> Box<dyn AsyncNodeLogic> {
        Box::new((*self).clone())
    }
}

/// The `AsyncBatchNode` factory
pub fn new_async_batch_node<L: AsyncNodeLogic + Clone>(logic: L) -> AsyncNode {
    AsyncNode::new(AsyncBatchLogic { logic })
}
