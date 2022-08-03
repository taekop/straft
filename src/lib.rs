#[macro_use]
extern crate slog;
#[macro_use]
extern crate async_trait;

pub mod rpc;
pub mod node;

pub type NodeId = String;
pub type Command = String;

#[derive(Debug)]
pub struct Entry {
    pub index: u64,
    pub term: u64,
    pub command: Command,
}
