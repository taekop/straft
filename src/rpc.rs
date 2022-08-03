use crate::Entry;
use crate::{Command, NodeId};
use anyhow::Result;

#[async_trait]
pub trait RPC {
    async fn append_entries(&self, request: AppendEntriesRequest) -> Result<AppendEntriesResponse>;
    async fn request_vote(&self, request: RequestVoteRequest) -> Result<RequestVoteResponse>;
    async fn append_log(&self, request: AppendLogRequest) -> Result<AppendLogResponse>;
}

#[derive(Debug)]
pub struct AppendEntriesRequest {
    pub term: u64,
    pub leader_id: NodeId,
    pub prev_log_index: u64,
    pub prev_log_term: u64,
    pub entries: Vec<Entry>,
    pub leader_commit: u64,
}

#[derive(Debug)]
pub struct AppendEntriesResponse {
    pub term: u64,
    pub success: bool,
}

#[derive(Debug)]
pub struct RequestVoteRequest {
    pub term: u64,
    pub candidate_id: NodeId,
    pub last_log_index: u64,
    pub last_log_term: u64,
}

#[derive(Debug)]
pub struct RequestVoteResponse {
    pub term: u64,
    pub vote_granted: bool,
}

#[derive(Debug)]
pub struct AppendLogRequest {
    pub command: Command,
}

#[derive(Debug)]
pub struct AppendLogResponse {
    pub success: bool,
    pub leader_id: Option<NodeId>,
    pub leader_address: Option<String>,
}
