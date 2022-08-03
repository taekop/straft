use anyhow::Result;
use logger::Logger;
use state::NodeState;

pub mod executor;
pub mod logger;
mod role;
mod state;

use crate::rpc::{
    AppendEntriesRequest, AppendEntriesResponse, AppendLogRequest, AppendLogResponse,
    RequestVoteRequest, RequestVoteResponse, RPC,
};
use crate::{Command, NodeId};
use executor::Executor;

pub struct Node<C: Command, E: Executor<C>> {
    id: NodeId,
    state: NodeState<C>,
    executor: E,
    logger: Logger,
}

impl<C: Command, E: Executor<C>> Node<C, E> {
    pub fn new(id: NodeId, executor: E, logger: Logger) -> Self {
        Node {
            id: id,
            state: NodeState::new(),
            executor: executor,
            logger: logger,
        }
    }
}

#[async_trait]
impl<C: Command, E: Executor<C>> RPC<C> for Node<C, E> {
    async fn append_entries(
        &self,
        request: AppendEntriesRequest<C>,
    ) -> Result<AppendEntriesResponse> {
        self.log_debug(format!("gRPC Request: AppendEntries {:?}", request));
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
