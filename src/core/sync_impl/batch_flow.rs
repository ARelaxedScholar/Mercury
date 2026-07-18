use crate::core::sync_impl::NodeValue;
use crate::core::sync_impl::node::{Node, NodeLogic};
use std::collections::HashMap;

/// An orchestratable node that runs a nested flow repeatedly with different parameters.
///
/// The nested flow is stored as a [`Node`] so batch flows can be composed and nested.
pub struct BatchFlow(Node);

/// The Derefs are needed to be able to access the inside `Node` of the `Flow` easily
impl std::ops::Deref for BatchFlow {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for BatchFlow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone)]
pub struct BatchFlowLogic<F>
where
    F: Fn(&HashMap<String, NodeValue>, &HashMap<String, NodeValue>) -> NodeValue
        + Clone
        + Send
        + Sync
        + 'static,
{
    // Storing a node permits both ordinary flows and nested batch flows.
    flow: Node,
    prep_fn: F,
}

impl<F> NodeLogic for BatchFlowLogic<F>
where
    F: Fn(&HashMap<String, NodeValue>, &HashMap<String, NodeValue>) -> NodeValue
        + Clone
        + Send
        + Sync
        + 'static,
{
    fn prep(
        &self,
        params: &HashMap<String, NodeValue>,
        shared: &HashMap<String, NodeValue>,
    ) -> NodeValue {
        // Call the user-defined closure
        serde_json::to_value((shared, (self.prep_fn)(params, shared)))
            .expect("serializing batch-flow shared state and parameters should succeed")
    }

    fn exec(&self, input: NodeValue) -> NodeValue {
        if let Some(array) = input.as_array() {
            if array.len() != 2 {
                panic!("BatchFlow input must contain shared state and batch parameters");
            }
            let mut shared: HashMap<String, NodeValue> =
                serde_json::from_value(array[0].clone()).unwrap_or_default();
            let params_array: Vec<HashMap<String, NodeValue>> =
                serde_json::from_value(array[1].clone()).unwrap_or_default();
            params_array.into_iter().for_each(|params| {
                let mut combined_params: HashMap<String, NodeValue> = params.clone();
                combined_params.extend(self.flow.data.params.clone());
                let mut flow = self.flow.clone();
                flow.set_params(combined_params);
                flow.run(&mut shared);
            });

            serde_json::to_value(shared).expect("serializing BatchFlow shared state should succeed")
        } else {
            panic!("BatchFlow input must be a two-element array");
        }
    }

    fn post(
        &self,
        shared: &mut HashMap<String, NodeValue>,
        _prep_res: NodeValue,
        exec_res: NodeValue,
    ) -> Option<String> {
        if let Ok(shared_post) = serde_json::from_value(exec_res) {
            *shared = shared_post
        } else {
            log::error!(
                "BatchFlow could not deserialize its result; shared state remains unchanged"
            );
        }
        // Returning the default route permits ordinary flow chaining.
        Some("default".into())
    }

    fn clone_box(&self) -> Box<dyn NodeLogic> {
        Box::new((*self).clone())
    }
}

impl BatchFlow {
    pub fn new<F>(flow: Node, prep_fn: F) -> Self
    where
        F: Fn(&HashMap<String, NodeValue>, &HashMap<String, NodeValue>) -> NodeValue
            + Clone
            + Send
            + Sync
            + 'static,
    {
        BatchFlow(Node::new(BatchFlowLogic { flow, prep_fn }))
    }
}
