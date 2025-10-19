use crate::core::async_impl::async_node::{AsyncNode, AsyncNodeLogic};
use crate::core::sync_impl::NodeValue;
use crate::core::sync_impl::node::{Node, NodeLogic};
use crate::core::{Executable, Executable::Async, Executable::Sync};
use std::collections::HashMap;

/// The logic that is specif
#[derive(Clone)]
pub struct AsyncFlowLogic {
    start: Executable,
}

/// A flow really, just is a Node with orchestration logic
/// to enforce that, we will create a NewType with a "factory" which prebuilds it.
#[derive(Clone)]
pub struct AsyncFlow(AsyncNode);

/// The Derefs are needed to be able to access the inside `Node` of the `Flow` easily
impl std::ops::Deref for AsyncFlow {
    type Target = AsyncNode;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for AsyncFlow {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsyncFlow {
    pub fn new(start: Executable) -> AsyncFlow {
        AsyncFlow(AsyncNode::new(AsyncFlowLogic { start }))
    }

    pub fn start(&mut self, start: Executable) {
        // extract the `NodeLogic` from the Flow
        let behaviour: &mut dyn AsyncNodeLogic = &mut *self.behaviour;

        if let Some(flow_logic) = behaviour.as_any_mut().downcast_mut::<AsyncFlowLogic>() {
            // Should always be possible if the Flow as created through the factory
            flow_logic.start = start;
        } else {
            // This should never happen, but somehow it did
            panic!("Error: Flow's logic is not of type FlowLogic");
        }
    }
}

impl AsyncNodeLogic for AsyncFlowLogic {
    async fn prep(
        &self,
        params: &HashMap<String, NodeValue>,
        shared: &HashMap<String, NodeValue>,
    ) -> NodeValue {
        serde_json::to_value((params, shared)).expect("If this works, I'll be so lit")
    }

    async fn exec(&self, input: NodeValue) -> NodeValue {
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
            last_action = match curr {
                Sync(sync_node) => {
                    sync_node.set_params(params.clone());
                    let mut shared_clone = shared.clone();

                    // not ideal, but not cloning here would
                    // require a significant refactoring
                    // afaik (switching everything to use
                    // Arc/Rc)
                    // Will be next step if benchmarking shows me this is actually
                    // worth the hassle
                    match tokio::task::spawn_blocking(move || {
                        let action = sync_node.run(&mut shared_clone).unwrap_or("default".into());
                        (action, shared_clone)
                    })
                    .await
                    {
                        Ok((next_action, modified_shared)) => {
                            // Happy path: the task completed successfully
                            *shared = modified_shared;
                            next_action;
                        }
                        Err(join_error) => {
                            // The background task panicked!
                            log::error!("A synchronous node panicked: {:?}", join_error);
                            // For now, just log it and go to the default action
                            "default".into();
                        }
                    }
                }
                Async(async_node) => {
                    async_node.set_params(params.clone());
                    async_node
                        .run(&mut shared)
                        .await
                        .unwrap_or("default".into())
                }
            };

            // Uses method implemented on Executable
            let next_executable = curr.successors().get(&last_action).cloned();

            match next_executable {
                Some(executable) => current = Some(executable),
                None => {
                    current = None;
                }
            }
        }
        serde_json::to_value((last_action.to_string(), shared))
            .expect("Serializing string and HashMap should be doable")
    }
    async fn post(
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
                    "Serialization into array succeeded, but got unexpected length for array: {}! Returning default values",
                    array.len()
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
