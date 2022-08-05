use anyhow::Result;
use logger::Logger;
use state::NodeState;
use tokio::time::{interval, Duration};

mod election_timer;
pub mod executor;
pub mod logger;
mod state;

use crate::rpc::{
    AppendEntriesRequest, AppendEntriesResponse, AppendLogRequest, AppendLogResponse,
    RequestVoteRequest, RequestVoteResponse, RPC,
};
use crate::{Command, NodeConfig};
use election_timer::ElectionTimer;
use executor::Executor;
use state::Role;

pub struct Node<C: Command, E: Executor<C>> {
    config: NodeConfig,
    state: NodeState<C>,
    election_timer: ElectionTimer,
    executor: E,
    logger: Logger,
}

impl<C: Command, E: Executor<C>> Node<C, E> {
    pub fn new(config: NodeConfig, executor: E, logger: Logger) -> Self {
        let election_timeout = config.election_timeout.clone();
        Node {
            config: config,
            state: NodeState::new(),
            election_timer: ElectionTimer::new(election_timeout),
            executor: executor,
            logger: logger,
        }
    }

    pub async fn start_heartbeat(&self) {
        self.log_info(format!("Start Heartbeat"));
        self.election_timer.reset();
        let mut interval = interval(Duration::from_millis(self.config.heartbeat_period));
        loop {
            interval.tick().await;
            self.heartbeat();
        }
    }

    pub fn heartbeat(&self) {
        self.log_debug(format!("Heartbeat"));
        if ((self.state.is_role(Role::FOLLOWER) || self.state.is_role(Role::CANDIDATE))
            && self.election_timer.is_election_timeout())
        {
            self.change_role(Role::CANDIDATE);
        } else if (self.state.is_role(Role::LEADER)) {
            self.request_append_entries();
        }
    }

    fn change_role(&self, role: Role) {
        self.log_debug(format!("Change role to {:?}", role));
        self.state.change_role(role);
        match role {
            Role::FOLLOWER => (),
            Role::CANDIDATE => {
                self.run_for_leader();
            },
            Role::LEADER => {
                self.state.initialize_leader_state(self.config.id.clone());
                self.request_append_entries();
            }
        }
    }

    fn run_for_leader(&self) {
        self.log_debug(format!("Run for leader in term {:?}", self.state.current_term));
    }

    fn request_append_entries(&self) {
        self.log_debug(format!("Request AppendEntries to other nodes"));
    }
}

#[async_trait]
impl<C: Command, E: Executor<C>> RPC<C> for Node<C, E> {
    async fn append_entries(
        &self,
        request: AppendEntriesRequest<C>,
    ) -> Result<AppendEntriesResponse> {
        self.log_debug(format!("gRPC Request: AppendEntries {:?}", request));
        self.election_timer.reset();
        Ok(AppendEntriesResponse {
            term: 0,
            success: false,
        })
    }

    async fn request_vote(&self, request: RequestVoteRequest) -> Result<RequestVoteResponse> {
        self.log_debug(format!("gRPC Request: RequestVote {:?}", request));
        Ok(RequestVoteResponse {
            term: 0,
            vote_granted: false,
        })
    }

    async fn append_log(&self, request: AppendLogRequest<C>) -> Result<AppendLogResponse> {
        self.log_debug(format!("gRPC Request: AppendLog {:?}", request));
        Ok(AppendLogResponse {
            success: false,
            leader_id: None,
            leader_address: None,
        })
    }
}
