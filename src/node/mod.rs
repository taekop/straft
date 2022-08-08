use anyhow::Result;
use futures::executor::block_on;
use logger::Logger;
use state::NodeState;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tokio::time::{interval, timeout, Duration};

mod election_timer;
pub mod executor;
pub mod logger;
mod state;

use crate::rpc::{
    AppendEntriesRequest, AppendEntriesResponse, AppendLogRequest, AppendLogResponse,
    RequestVoteRequest, RequestVoteResponse, RPC,
};
use crate::{Command, NodeClient, NodeConfig};
use election_timer::ElectionTimer;
use executor::Executor;
use state::Role;

pub struct Node<C: Command, E: Executor<C>, Client: NodeClient<C>> {
    config: NodeConfig<C, Client>,
    state: NodeState<C>,
    election_timer: ElectionTimer,
    executor: E,
    logger: Logger,
}

impl<C: Command, E: Executor<C>, Client: NodeClient<C>> Node<C, E, Client> {
    pub fn new(config: NodeConfig<C, Client>, executor: E, logger: Logger) -> Self {
        let election_timeout = config.election_timeout.clone();
        Node {
            config: config,
            state: NodeState::new(),
            election_timer: ElectionTimer::new(election_timeout),
            executor: executor,
            logger: logger,
        }
    }

    pub fn start_heartbeat(&self) {
        self.log_info(format!("Start Heartbeat"));
        self.election_timer.reset();
        let mut interval = interval(Duration::from_millis(self.config.heartbeat_period));
        loop {
            block_on(interval.tick());
            self.heartbeat();
        }
    }

    pub fn heartbeat(&self) {
        self.log_debug(format!("Heartbeat"));
        if self.state.is_role(Role::FOLLOWER)
            || self.state.is_role(Role::CANDIDATE) && self.election_timer.is_election_timeout()
        {
            self.change_role(Role::CANDIDATE);
        } else if self.state.is_role(Role::LEADER) {
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
            }
            Role::LEADER => {
                self.state.initialize_leader_state(self.config.id.clone());
                self.request_append_entries();
            }
        }
    }

    fn run_for_leader(&self) {
        let lock_current_term = self.state.current_term.try_lock();
        if let Ok(mut current_term) = lock_current_term {
            self.log_debug(format!("Run for leader in term {:?}", *current_term));
            *current_term += 1;
            *block_on(self.state.voted_for.lock()) = Some(self.config.id.clone());
            let vote_cnt = Arc::new(AtomicU64::new(1));
            let max_term = Arc::new(AtomicU64::new(0));
            let msg = RequestVoteRequest {
                term: *current_term,
                candidate_id: self.config.id.clone(),
                last_log_index: self.state.log.last().unwrap().index,
                last_log_term: self.state.log.last().unwrap().term,
            };

            let mut handles = vec![];
            self.config.client.iter().for_each(|(follower_id, client)| {
                let client = client.clone();
                let vote_cnt = Arc::clone(&vote_cnt);
                let max_term = Arc::clone(&max_term);
                let msg = msg.clone();
                handles.push(async move {
                    timeout(
                        Duration::from_millis(self.election_timer.until_next_timeout()),
                        async move {
                            let client = client.lock().await;
                            let res = client.request_vote(msg).await.unwrap();
                            drop(client);
                            self.log_debug(format!(
                                "Get RequestVoteResponse from Node#{:?}: {:?}",
                                follower_id, res
                            ));
                            let (vote_granted, term) = (res.vote_granted, res.term);
                            if vote_granted {
                                vote_cnt.fetch_add(1, Ordering::Relaxed);
                            }
                            max_term.fetch_max(term, Ordering::Relaxed);
                        },
                    )
                });
            });
            block_on(futures::future::join_all(handles));
        }
    }

    fn request_append_entries(&self) {
        self.log_debug(format!("Request AppendEntries to other nodes"));
    }
}

#[async_trait]
impl<C: Command, E: Executor<C>, Client: NodeClient<C>> RPC<C> for Node<C, E, Client> {
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
