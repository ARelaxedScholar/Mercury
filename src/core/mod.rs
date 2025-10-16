mod sync_impl;
mod async_impl;

use sync_impl::node::Node;
use async_impl::async_node::AsyncNode;

/// The General Executable Enum
pub enum Executable {
    Sync(Node),
    Async(AsyncNode),
}
