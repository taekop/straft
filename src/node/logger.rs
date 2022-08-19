extern crate slog;

use crate::node::client::ExternalNodeClient;
use crate::node::Node;
use crate::node::StateMachineClient;

pub type Logger = slog::Logger;

impl<SM: StateMachineClient, Client: ExternalNodeClient> Node<SM, Client> {
    pub fn log_trace(&self, msg: String) {
        let role = self.state.role();
        trace!(self.logger, "[{}] {}", role, msg);
    }

    pub fn log_debug(&self, msg: String) {
        let role = self.state.role();
        debug!(self.logger, "[{}] {}", role, msg);
    }

    pub fn log_info(&self, msg: String) {
        let role = self.state.role();
        info!(self.logger, "[{}] {}", role, msg);
    }

    pub fn log_warning(&self, msg: String) {
        let role = self.state.role();
        warn!(self.logger, "[{}] {}", role, msg);
    }

    pub fn log_error(&self, msg: String) {
        let role = self.state.role();
        error!(self.logger, "[{}] {}", role, msg);
    }

    pub fn log_critical(&self, msg: String) {
        let role = self.state.role();
        crit!(self.logger, "[{}] {}", role, msg);
    }
}
