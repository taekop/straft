extern crate slog_stdlog;

mod entry;
mod logger;
mod role;
mod server;
mod state;

use crate::NodeId;
use logger::Logger;
use state::NodeState;

pub struct Node {
    id: NodeId,
    state: NodeState,
    logger: Logger,
}

impl Node {
    pub fn new<L: Into<Option<Logger>>>(id: NodeId, logger: L) -> Self {
        Node {
            id: id,
            state: NodeState::new(),
            logger: logger.into().unwrap_or(logger::default()),
        }
    }
}
