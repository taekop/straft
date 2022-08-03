#[macro_use]
extern crate slog;
#[macro_use]
extern crate async_trait;

use std::fmt::Debug;

pub mod rpc;
pub mod node;

pub type NodeId = String;
pub trait Command: Send + Sync + Debug {}

#[derive(Debug)]
pub struct Entry<C: Command> {
    pub index: u64,
    pub term: u64,
    pub command: C,
}

pub use node::executor::Executor;
pub use node::logger;
