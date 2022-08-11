use crate::Entry;
use crate::{Command, NodeId};
use anyhow::Result;

pub trait RPC<C: Command> {
    fn append_entries(&mut self, request: AppendEntriesRequest<C>)
        -> Result<AppendEntriesResponse>;
    fn request_vote(&mut self, request: RequestVoteRequest) -> Result<RequestVoteResponse>;
    fn append_log(&mut self, request: AppendLogRequest<C>) -> Result<AppendLogResponse>;
}

pub trait RPCClient<C: Command>: 'static + RPC<C> + Clone + Send {}

#[derive(Debug, Clone)]
pub struct AppendEntriesRequest<C: Command> {
    pub term: u64,
    pub leader_id: NodeId,
    pub prev_log_index: usize,
    pub prev_log_term: u64,
    pub entries: Vec<Entry<C>>,
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
pub struct AppendLogRequest<C: Command> {
    pub command: C,
}

#[derive(Debug, Clone)]
pub struct AppendLogResponse {
    pub success: bool,
    pub leader_id: Option<NodeId>,
    pub leader_address: Option<String>,
}
