#[macro_use]
extern crate slog;

use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Range;

pub mod node;
pub mod rpc;
pub mod state_machine;

pub use node::logger::Logger;
pub use node::{
    actor::{RequestMessage, ResponseMessage},
    Node,
};
pub use rpc::{RPCClient, RPC};
pub use state_machine::StateMachineClient;

pub type NodeId = String;
pub trait Command: 'static + Debug + Default + Clone + Send + Sync {}

pub struct NodeConfig<C: Command, Client: RPCClient<C>> {
    pub id: NodeId,
    pub addresses: HashMap<NodeId, String>,
    pub client: HashMap<NodeId, Client>,
    pub election_timeout: Range<u64>,
    pub heartbeat_period: u64,
    pub majority: u64,
    pub _phantom_c: PhantomData<C>,
}

#[derive(Debug, Clone)]
pub struct Entry<C: Command> {
    pub index: usize,
    pub term: u64,
    pub command: C,
}
