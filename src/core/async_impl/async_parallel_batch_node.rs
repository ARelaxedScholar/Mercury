use crate::core::async_impl::async_node::{AsyncNode, AsyncNodeLogic};
use crate::core::sync_impl::NodeValue;
use async_trait::async_trait;
use futures::stream::{FuturesOrdered, StreamExt};
use std::collections::HashMap;

const DEFAULT_MAX_CONCURRENCY: usize = 50;

#[derive(Clone)]
pub struct AsyncParallelBatchLogic<L: AsyncNodeLogic> {
    logic: L,
    max_concurrency: usize,
}

impl<L: AsyncNodeLogic> AsyncParallelBatchLogic<L> {
    pub fn new(logic: L) -> Self {
        AsyncParallelBatchLogic {
            logic,
            max_concurrency: DEFAULT_MAX_CONCURRENCY,
        }
    }

    pub fn with_concurrency(self, max_concurrency: usize) -> Self {
        assert!(
            max_concurrency > 0,
            "Max concurrency must be greater than 0"
        );
        AsyncParallelBatchLogic {
            logic: self.logic,
            max_concurrency,
        }
    }
}

#[async_trait]
impl<L: AsyncNodeLogic + Clone> AsyncNodeLogic for AsyncParallelBatchLogic<L> {
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
            let mut results: Vec<NodeValue> = Vec::new();

            let futures = arr.into_iter().map(|item| self.logic.exec(item.clone()));
            let mut futures_ordered : FuturesOrdered<_>= futures.collect();

            while let Some(result) = futures_ordered.next().await {
                results.push(result);
            }
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
pub fn new_async_parallel_batch_node<L: AsyncNodeLogic + Clone>(
    logic: AsyncParallelBatchLogic<L>,
) -> AsyncNode {
    AsyncNode::new(logic)
}
