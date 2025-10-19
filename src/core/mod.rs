mod async_impl;
mod sync_impl;

use async_impl::async_node::AsyncNode;
use std::collections::HashMap;
use sync_impl::node::Node;

use crate::core::sync_impl::NodeValue;

/// The General Executable Enum
pub enum Executable {
    Sync(Node),
    Async(AsyncNode),
}

impl Executable {
    pub fn successors(&self) -> &HashMap<String, NodeValue> {
        match self {
            Executable::Sync(node) | Executable::Async(node) => &node.data.successors,
        }
    }
}
