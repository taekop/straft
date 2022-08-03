extern crate slog_stdlog;

mod logger;
mod role;
mod state;

use crate::rpc::{
    AppendEntriesRequest, AppendEntriesResponse, AppendLogRequest, AppendLogResponse,
    RequestVoteRequest, RequestVoteResponse, RPC,
};
use crate::NodeId;
use anyhow::Result;
use logger::Logger;
use state::NodeState;

pub struct Node {
    id: NodeId,
    state: NodeState,
    logger: Logger,
}

impl Node {
    pub fn new<L: Into<Option<Logger>>>(id: NodeId, logger: L) -> Self {
        Node {
            id: id,
            state: NodeState::new(),
            logger: logger.into().unwrap_or(logger::default()),
        }
    }
}

#[async_trait]
impl RPC for Node {
    async fn append_entries(&self, request: AppendEntriesRequest) -> Result<AppendEntriesResponse> {
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

    async fn append_log(&self, request: AppendLogRequest) -> Result<AppendLogResponse> {
        self.log_debug(&format!("gRPC Request: AppendLog {:?}", request));
        Ok(AppendLogResponse {
            success: false,
            leader_id: None,
            leader_address: None,
        })
    }
}
