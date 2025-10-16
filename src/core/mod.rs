mod async_impl;
mod sync_impl;

use async_impl::async_node::AsyncNode;
use sync_impl::node::Node;

/// The General Executable Enum
pub enum Executable {
    Sync(Node),
    Async(AsyncNode),
}
