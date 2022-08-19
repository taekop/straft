#[macro_use]
extern crate slog;

use std::collections::HashSet;
use std::fmt::Debug;

pub mod node;
pub mod rpc;
pub mod state_machine;

pub use node::logger::Logger;
pub use node::{
    actor::{RequestMessage, ResponseMessage},
    client::{ExternalNodeClient, InternalNodeClient},
    Node,
};
pub use state_machine::StateMachineClient;

pub type NodeId = String;

pub struct ClusterConfig {
    pub heartbeat_period: u64,
    pub minimum_election_timeout: u64,
    pub maximum_election_timeout: u64,
    pub snapshot_threshold: usize,
    pub snapshot_chunk_size: usize,
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
    ChangeMembership(Option<HashSet<NodeId>>, Option<HashSet<NodeId>>), // new_members, non_voting_members, doesn't change if none
}
