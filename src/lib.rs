#[macro_use]
extern crate slog;

use anyhow::Result;
use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Range;
use std::sync::mpsc::SyncSender;

pub mod node;
pub mod rpc;
pub mod state_machine;

pub use node::logger::Logger;
pub use node::{
    actor::{RequestMessage, ResponseMessage},
    client::NodeClient,
    Node,
};
pub use rpc::{RPCClient, RPC};
pub use state_machine::StateMachineClient;

pub type NodeId = String;

pub struct NodeConfig<Client: RPCClient> {
    pub id: NodeId,
    pub addresses: HashMap<NodeId, String>,
    pub client: HashMap<NodeId, Client>,
    pub election_timeout: Range<u64>,
    pub heartbeat_period: u64,
    pub majority: u64,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub index: usize,
    pub term: u64,
    pub command: String,
    // send response when applied to state machine
    pub sender: Option<SyncSender<Result<String>>>,
}
