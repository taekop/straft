#[macro_use]
extern crate slog;

use std::collections::HashSet;
use std::fmt::Debug;
use std::ops::Range;

pub mod node;
pub mod rpc;
pub mod state_machine;

pub use node::logger::Logger;
pub use node::{
    actor::{RequestMessage, ResponseMessage},
    client::{InternalNodeClient, ExternalNodeClient},
    Node,
};
pub use state_machine::StateMachineClient;

pub type NodeId = String;

pub struct ClusterConfig {
    pub members: HashSet<NodeId>,
    pub election_timeout: Range<u64>,
    pub heartbeat_period: u64,
    pub majority: u64,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub index: usize,
    pub term: u64,
    pub command: Command,
}

#[derive(Debug, Clone)]
pub enum Command {
    Empty,
    Write(String),
    ChangeMembership(HashSet<NodeId>, HashSet<NodeId>),
}
