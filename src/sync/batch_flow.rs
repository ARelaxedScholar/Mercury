use crate::sync::node::{Node, NodeLogic};
use crate::sync::{NodeValue, flow::Flow};
use std::collections::HashMap;

/// A BatchFlow is a `Node` (so orchestrable) which runs
/// a `Flow` many times with different params.
/// Therefore, a `BatchFlow` must deref the `Node`
/// But it's actual exectution logic should be implemented
/// on a struct which carries the `Flow` to batch.
pub struct BatchFlow(Node);

#[derive(Clone)]
pub struct BatchFlowLogic<F>
where
    F: Fn(&HashMap<String, NodeValue>, &HashMap<String, NodeValue>) -> NodeValue
        + Clone
        + Send
        + Sync
        + 'static,
{
    flow: Flow,
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
            .expect("Serialization of shared to thing should work")
    }

    fn exec(&self, input: NodeValue) -> NodeValue {
        if let Some(array) = input.as_array() {
            if array.len() != 2 {
                panic!("Well shit");
            }
            // Ok, we covered our bases now
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

            NodeValue::Null
        } else {
            panic!("Serialization failure occured");
        }
    }

    fn post(
        &self,
        _shared: &mut HashMap<String, NodeValue>,
        _prep_res: NodeValue,
        _exec_res: NodeValue,
    ) -> Option<String> {
        // In PocketFlow they return the exec_res, but I think it's cleaner like this. If
        // you're not happy with this, you can also just implement your custom
        // BatchFlowLogic
        None
    }

    fn clone_box(&self) -> Box<dyn NodeLogic> {
        Box::new((*self).clone())
    }
}

impl BatchFlow {
    pub fn new<F>(flow: Flow, prep_fn: F) -> Self
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
