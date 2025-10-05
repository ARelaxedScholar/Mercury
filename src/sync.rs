use std::collections::HashMap;

/// The Alias for serde_json::Value since I use it a lot
pub type NodeValue = serde_json::Value;

/// ------ Base Node Logic -------------------------------------------------------
/// Defines the fundamental logic that is common to any "Node" of the system
#[derive(Clone)]
pub struct Node {
    data: NodeCore,
    behaviour: Box<dyn NodeLogic>,
}

impl Node {
    fn new<L: NodeLogic + 'static>(behaviour: L) -> Self {
        Node {
            data: NodeCore::default(),
            behaviour: Box::new(behaviour),
        }
    }
    fn set_params(&mut self, params: HashMap<String, NodeValue>) {
        self.data.params = params;
    }
    fn next(self, node: Node) -> Self {
        self.next_on(node, "default")
    }
    fn next_on(mut self, node: Node, action: &str) -> Self {
        if self.data.successors.contains_key(action) {
            log::warn!(
                "{}",
                format!(
                    "Warning: Action {} was found in successors, Overwriting key {}.",
                    &action, &action
                )
            );
        }
        self.data.successors.insert(action.to_string(), node);
        self
    }

    fn run(&self, shared: &mut HashMap<String, NodeValue>) -> Option<String> {
        let p = self.behaviour.prep(&self.data.params, shared);
        let e = self.behaviour.exec(p.clone());
        self.behaviour.post(shared, p, e)
    }
}

#[derive(Default, Clone)]
pub struct NodeCore {
    params: HashMap<String, NodeValue>,
    successors: HashMap<String, Node>,
}

pub trait NodeLogic: Send + Sync + 'static {
    fn prep(&self, _params: &HashMap<String, NodeValue>, _shared: &HashMap<String, Any>) -> Any {
        NodeValue::default()
    }
    fn exec(&self, _input: NodeValue) -> Any {
        NodeValue::default()
    }
    fn post(
        &self,
        _shared: &mut HashMap<String, NodeValue>,
        _prep_res: NodeValue,
        _exec_res: NodeValue,
    ) -> Option<String> {
        None
    }

    fn clone_box(&self) -> Box<dyn NodeLogic>;
}

impl Clone for Box<dyn NodeLogic> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

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
    fn new(logic: L) -> Self {
        BatchLogic { logic }
    }
}

/// The advent of the BatchNode
/// Defining the logic for what is a `BatchLogic` which is a "true" `NodeLogic`.
impl<L: NodeLogic + Clone> NodeLogic for BatchLogic<L> {
    fn prep(&self, params: &HashMap<String, NodeValue>, shared: &HashMap<String, Any>) -> Any {
        self.logic.prep(params, shared)
    }

    fn exec(&self, items: NodeValue) -> Any {
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

/// With our new composable approach we don't need the "awkward" workaround that Pocketflow
/// has where they use an orch method instead of the exec method. A Flow is not "special"
/// it's just a Node who's logic encapsulates other Nodes, changing the `exec` method
/// to encapsulate that is possible but awkward. Instead, we can simply define aliases
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
}
