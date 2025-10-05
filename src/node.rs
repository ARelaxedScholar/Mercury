use std::collections::HashMap;

#[derive(Clone, Debug, Default)]
pub struct ExecInput {
    value: serde_json::Value,
}

#[derive(Clone, Debug, Default)]
pub struct ExecOutput {
    value: serde_json::Value,
}

pub struct NodeCore {
    params: HashMap<String, serde_json::Value>,
    successors: HashMap<String, Box<dyn Node>>,
}

pub struct ConditionalTransition {
    node: Box<dyn Node>,
    action: String,
}

pub trait Node {
    fn core_ref(&self) -> &NodeCore;
    fn core_mut_ref(&mut self) -> &mut NodeCore;
    fn set_params(&mut self, params: HashMap<String, serde_json::Value>) {
        self.core_mut_ref().params = params;
    }
    fn next(&mut self, node: Box<dyn Node>, action: Option<String>) -> Box<dyn Node> {
        let action: String = action.unwrap_or("default".into());
        if self.core_ref().successors.contains_key(&action) {
            log::warn!(
                "{}",
                format!(
                    "Warning: Action {} was found in successors, Overwriting key {}.",
                    &action, &action
                )
            );
        }
        self.core_mut_ref().successors.insert(action, node.clone());
        node
    }
    fn prep(&self, _shared: &mut HashMap<String, serde_json::Value>) -> ExecInput {
        ExecInput::default()
    }
    fn exec(&self, _input: ExecInput) -> ExecOutput {
        ExecOutput::default()
    }
    fn post(
        &self,
        _shared: &mut HashMap<String, serde_json::Value>,
        _prep_res: ExecInput,
        _exec_res: ExecOutput,
    ) -> Option<String> {
        None
    }
    fn run(&self, shared: &mut HashMap<String, serde_json::Value>) -> Option<String> {
        let p = self.prep(shared);
        let e = self.exec(p.clone());
        self.post(shared, p, e)
    }

    fn clone_box(&self) -> Box<dyn Node>;
}

impl Clone for Box<dyn Node> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
