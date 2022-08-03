extern crate slog_stdlog;

use anyhow::Result;
use logger::Logger;
use state::NodeState;

mod logger;
mod role;
mod state;

use crate::rpc::{
    AppendEntriesRequest, AppendEntriesResponse, AppendLogRequest, AppendLogResponse,
    RequestVoteRequest, RequestVoteResponse, RPC,
};
use crate::{Command, NodeId};

pub struct Node<C: Command> {
    id: NodeId,
    state: NodeState<C>,
    logger: Logger,
}

impl<C: Command> Node<C> {
    pub fn new<L: Into<Option<Logger>>>(id: NodeId, logger: L) -> Self {
        Node {
            id: id,
            state: NodeState::new(),
            logger: logger.into().unwrap_or(logger::default()),
        }
    }
}

#[async_trait]
impl<C: Command> RPC<C> for Node<C> {
    async fn append_entries(&self, request: AppendEntriesRequest<C>) -> Result<AppendEntriesResponse> {
        self.log_debug(&format!("gRPC Request: AppendEntries {:?}", request));
        Ok(AppendEntriesResponse {
            term: 0,
            success: false,
        })
    }

    async fn request_vote(&self, request: RequestVoteRequest) -> Result<RequestVoteResponse> {
        self.log_debug(&format!("gRPC Request: RequestVote {:?}", request));
        Ok(RequestVoteResponse {
            term: 0,
            vote_granted: false,
        })
    }

    async fn append_log(&self, request: AppendLogRequest<C>) -> Result<AppendLogResponse> {
        self.log_debug(&format!("gRPC Request: AppendLog {:?}", request));
        Ok(AppendLogResponse {
            success: false,
            leader_id: None,
            leader_address: None,
        })
    }
}
