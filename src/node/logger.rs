extern crate slog;

use crate::node::executor::Executor;
use crate::node::Node;
use crate::{Command, NodeClient};

pub type Logger = slog::Logger;

impl<C: Command, E: Executor<C>, Client: NodeClient<C>> Node<C, E, Client> {
    pub fn log_debug(&self, msg: String) {
        debug!(self.logger, "{}", msg);
    }

    pub fn log_info(&self, msg: String) {
        info!(self.logger, "{}", msg);
    }
}
