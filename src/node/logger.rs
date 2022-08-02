extern crate slog;

use crate::node::Node;
use slog::Drain;

pub type Logger = slog::Logger;

pub fn default() -> Logger {
    slog::Logger::root(slog_stdlog::StdLog.fuse(), o!())
}

impl Node {
    pub fn log_debug(&self, msg: &str) {
        debug!(self.logger, "{}", msg);
    }

    pub fn log_info(&self, msg: &str) {
        info!(self.logger, "{}", msg);
    }
}
