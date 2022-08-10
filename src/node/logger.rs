extern crate slog;

use crate::node::Node;
use crate::node::StateMachineClient;
use crate::rpc::RPCClient;
use crate::Command;

pub type Logger = slog::Logger;

impl<C: Command, SM: StateMachineClient<C>, Client: RPCClient<C>> Node<C, SM, Client> {
    pub fn log_trace(&self, msg: String) {
        trace!(self.logger, "{}", msg);
    }

    pub fn log_debug(&self, msg: String) {
        debug!(self.logger, "{}", msg);
    }

    pub fn log_info(&self, msg: String) {
        info!(self.logger, "{}", msg);
    }

    pub fn log_warning(&self, msg: String) {
        warn!(self.logger, "{}", msg);
    }

    pub fn log_error(&self, msg: String) {
        error!(self.logger, "{}", msg);
    }

    pub fn log_critical(&self, msg: String) {
        crit!(self.logger, "{}", msg);
    }
}
