#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate slog;
extern crate tokio;

use std::marker::PhantomData;
use std::ops::Range;
use std::sync::Arc;
use std::{collections::HashMap, fmt::Debug};
use tokio::sync::Mutex;

pub mod node;
pub mod rpc;

pub use node::{
    executor::Executor,
    logger::{self, Logger},
    Node,
};
use rpc::RPC;

pub type NodeId = String;
pub trait Command: Send + Sync + Debug + Default {}
pub trait NodeClient<C: Command>: RPC<C> + Send + Sync {}

pub struct NodeConfig<C: Command, Client: NodeClient<C>> {
    pub id: NodeId,
    pub client: HashMap<NodeId, Arc<Mutex<Client>>>,
    pub election_timeout: Range<u64>,
    pub heartbeat_period: u64,
    pub majority: u64,
    pub _phantom_c: PhantomData<C>,
}

#[derive(Debug)]
pub struct Entry<C: Command> {
    pub index: u64,
    pub term: u64,
    pub command: C,
}
