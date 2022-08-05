#[macro_use]
extern crate slog;
#[macro_use]
extern crate async_trait;

use std::fmt::Debug;
use std::ops::Range;

pub mod node;
pub mod rpc;

pub type NodeId = String;
pub trait Command: Send + Sync + Debug {}

pub struct NodeConfig {
    pub id: NodeId,
    pub election_timeout: Range<u128>,
    pub heartbeat_period: u64,
}

#[derive(Debug)]
pub struct Entry<C: Command> {
    pub index: u64,
    pub term: u64,
    pub command: C,
}

pub use node::{
    executor::Executor,
    logger::{self, Logger},
    Node,
};
