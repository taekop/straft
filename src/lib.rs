#[macro_use]
extern crate slog;

pub mod node;

pub type NodeId = &'static str;
pub type Command = String;
