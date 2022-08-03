extern crate slog;

use slog::Drain;

use crate::node::Node;
use crate::Command;

pub type Logger = slog::Logger;

pub fn default() -> Logger {
    slog::Logger::root(slog_stdlog::StdLog.fuse(), o!())
}

impl<C: Command> Node<C> {
    pub fn log_debug(&self, msg: &str) {
        debug!(self.logger, "{}", msg);
    }

    pub fn log_info(&self, msg: &str) {
        info!(self.logger, "{}", msg);
    }
}
