use crate::Entry;
use crate::{Command, NodeId};
use anyhow::Result;

pub trait RPC {
    fn append_entries(&mut self, request: AppendEntriesRequest) -> Result<AppendEntriesResponse>;
    fn request_vote(&mut self, request: RequestVoteRequest) -> Result<RequestVoteResponse>;
    fn append_log(&mut self, request: AppendLogRequest) -> Result<AppendLogResponse>;
}

pub trait RPCClient: 'static + RPC + Clone + Send {}

#[derive(Debug, Clone)]
pub struct AppendEntriesRequest {
    pub term: u64,
    pub leader_id: NodeId,
    pub prev_log_index: usize,
    pub prev_log_term: u64,
    pub entries: Vec<Entry>,
    pub leader_commit: usize,
}

#[derive(Debug, Clone)]
pub struct AppendEntriesResponse {
    pub term: u64,
    pub success: bool,
}

#[derive(Debug, Clone)]
pub struct RequestVoteRequest {
    pub term: u64,
    pub candidate_id: NodeId,
    pub last_log_index: usize,
    pub last_log_term: u64,
}

#[derive(Debug, Clone)]
pub struct RequestVoteResponse {
    pub term: u64,
    pub vote_granted: bool,
}

#[derive(Debug, Clone)]
pub struct AppendLogRequest {
    pub command: Command,
}

#[derive(Debug, Clone)]
pub struct AppendLogResponse {
    pub success: bool,
    pub leader_id: Option<NodeId>,
    pub leader_address: Option<String>,
}
