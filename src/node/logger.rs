extern crate slog;

use crate::node::executor::Executor;
use crate::node::Node;
use crate::Command;

pub type Logger = slog::Logger;

impl<C: Command, E: Executor<C>> Node<C, E> {
    pub fn log_debug(&self, msg: &str) {
        debug!(self.logger, "{}", msg);
    }

    pub fn log_info(&self, msg: &str) {
        info!(self.logger, "{}", msg);
    }
}
