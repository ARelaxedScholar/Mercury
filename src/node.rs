use std::collections::HashMap;

/// The Alias for serde_json::Value since I use it a lot
pub type Any = serde_json::Value;

/// ------ Base Node -------------------------------------------------------
/// Defines the fundamental logic that is common to any "Node" of the system
#[derive(Clone, Debug, Default)]
pub struct ExecInput {
    value: Any,
}

#[derive(Clone, Debug, Default)]
pub struct ExecOutput {
    value: Any,
}

#[derive(Clone)]
pub struct Node {
    data: NodeCore,
    behaviour: Box<dyn NodeLogic>,
}

impl Node {
    fn set_params(&mut self, params: HashMap<String, Any>) {
        self.data.params = params;
    }
    fn next(mut self, node: Node, action: Option<String>) -> Self {
        let action: String = action.unwrap_or("default".into());
        if self.data.successors.contains_key(&action) {
            log::warn!(
                "{}",
                format!(
                    "Warning: Action {} was found in successors, Overwriting key {}.",
                    &action, &action
                )
            );
        }
        self.data.successors.insert(action, node.clone());
        self
    }

    fn run(&self, shared: &mut HashMap<String, Any>) -> Option<String> {
        let p = self.behaviour.prep(&self.data.params, shared);
        let e = self.behaviour.exec(p.clone());
        self.behaviour.post(shared, p, e)
    }
}


#[derive(Clone)]
pub struct NodeCore {
    params: HashMap<String, Any>,
    successors: HashMap<String, Node>,
}

pub trait NodeLogic: Send + Sync {
    fn prep(&self, _params: &HashMap<String, Any>, _shared: &mut HashMap<String, serde_json::Value>) -> ExecInput {
        ExecInput::default()
    }
    fn exec(&self, _input: ExecInput) -> ExecOutput {
        ExecOutput::default()
    }
    fn post(
        &self,
        _shared: &mut HashMap<String, Any>,
        _prep_res: ExecInput,
        _exec_res: ExecOutput,
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

