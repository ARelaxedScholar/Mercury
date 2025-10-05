use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ExecInput {
    value: serde_json::Value,
}

#[derive(Clone, Debug)]
pub struct ExecOutput {
    value: serde_json::Value,
}

pub struct NodeCore {
    params: HashMap<String, serde_json::Value>,
    successors: HashMap<String, Box<dyn Node>>,
}

impl Node for NodeCore {
    

}


impl std::ops::Shr<NodeCore> for &mut NodeCore {
    type Output = Self;
    
    fn shr(self, rhs: &mut NodeCore) -> Self::Output {
        self.successors.insert("default".into(), Box::new(rhs));
        rhs
    }
} 

pub struct ConditionalTransition {
    node: NodeCore,
    action: String,
}

pub trait Node {
    fn set_params(&mut self, params: HashMap<String, serde_json::Value>);
    fn next(&mut self, node: Box<dyn Node>, action: Option<String>) -> Box<dyn Node>;
    fn prep(&self, shared: HashMap<String, serde_json::Value>) -> ExecInput;
    fn exec(&self, input: ExecInput) -> ExecOutput;
    fn post(&self, shared: HashMap<String, serde_json::Value>, prep_res: ExecInput, exec_res: ExecOutput) -> String;
    fn run(&self, shared: HashMap<String, serde_json::Value>) -> String {
        let p = self.prep(shared.clone());
        let e = self.exec(p.clone());
        self.post(shared.clone(), p, e)
    }
}

