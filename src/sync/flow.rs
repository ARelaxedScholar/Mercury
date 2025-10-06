use std::collections::HashMap;
use crate::sync::NodeValue;
use crate::sync::node::{Node, NodeLogic};

/// With our new composable approach we don't need the "awkward" workaround that Pocketflow
/// has where they use an orch method instead of the exec method. A Flow is not "special"
/// it's just a Node who's logic encapsulates other Nodes, changing the `exec` method
/// to encapsulate that is possible but awkward. Instead, we can simply define aliases
#[derive(Clone)]
pub struct FlowLogic {
    start: Node,
}

/// A flow really, just is a Node with orchestration logic
/// to enforce that, we will create a NewType with a "factory" which prebuilds it.
#[derive(Clone)]
pub struct Flow(Node);

/// The Derefs are needed to be able to access the inside `Node` of the `Flow` easily
impl std::ops::Deref for Flow {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Flow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Flow {
    pub fn new(start: Node) -> Flow {
        Flow(Node::new(FlowLogic { start }))
    }

    pub fn start(&mut self, start: Node) {
        // extract the `NodeLogic` from the Flow
        let behaviour: &mut dyn NodeLogic = &mut *self.behaviour;

        if let Some(flow_logic) = behaviour.as_any_mut().downcast_mut::<FlowLogic>() {
            // Should always be possible if the Flow as created through the factory
            flow_logic.start = start;
        } else {
            // This should never happen, but somehow it did
            panic!("Error: Flow's logic is not of type FlowLogic");
        }
    }
}

impl NodeLogic for FlowLogic {
    fn prep(
        &self,
        params: &HashMap<String, NodeValue>,
        shared: &HashMap<String, NodeValue>,
    ) -> NodeValue {
        serde_json::to_value((params, shared)).expect("If this works, I'll be so lit")
    }
    fn exec(&self, input: NodeValue) -> NodeValue {
        //  This is the init (Basically, we deserialize the value that was passed from the previous
        //  step)
        let (params, mut shared): (HashMap<String, NodeValue>, HashMap<String, NodeValue>) =
            if let Some(arr) = input.as_array() {
                if arr.len() != 2 {
                    log::error!("serde_json::to_value() failed to convert the params and shared.");
                    (HashMap::new(), HashMap::new())
                } else {
                    // We can proceed
                    let params = serde_json::from_value(arr[0].clone()).unwrap_or_default();
                    let shared = serde_json::from_value(arr[1].clone()).unwrap_or_default();
                    (params, shared)
                }
            } else {
                (HashMap::new(), HashMap::new())
            };
        let mut current: Option<Node> = Some(self.start.clone());
        let mut last_action: String = "".into();

        // This is the orchestration logic
        while let Some(mut curr) = current {
            curr.set_params(params.clone());
            last_action = curr.run(&mut shared).unwrap_or("default".into());
            current = curr.data.successors.get(&last_action).cloned();
        }
        serde_json::to_value((last_action.to_string(), shared))
            .expect("Serializing string and HashMap should be doable")
    }
    fn post(
        &self,
        shared: &mut HashMap<String, NodeValue>,
        _prep_res: NodeValue,
        exec_res: NodeValue,
    ) -> Option<String> {
        let (last_action, shared_post): (String, HashMap<String, NodeValue>) = if let Some(array) =
            exec_res.as_array()
        {
            if array.len() != 2 {
                log::error!(
                    "{}",
                    format!(
                        "Serialization into array succeeded, but got unexpected length for array: {}! Returning default values",
                        array.len()
                    )
                );
                ("default".into(), shared.clone())
            } else {
                // happy path
                let last_action = serde_json::from_value(array[0].clone()).unwrap_or_default();
                let shared = serde_json::from_value(array[1].clone()).unwrap_or_default();
                (last_action, shared)
            }
        } else {
            log::error!("Serialization of last action and shared failed! Returning default values");
            ("default".into(), shared.clone())
        };

        // modify the shared state
        *shared = shared_post;

        // return the final action (since Flow is also just a node)
        Some(last_action)
    }
    fn clone_box(&self) -> Box<dyn NodeLogic> {
        Box::new((*self).clone())
    }
}
