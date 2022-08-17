use std::collections::HashSet;

use crate::{Entry, NodeId};

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
pub struct ChangeMembershipRequest {
    pub members: HashSet<NodeId>,
}

#[derive(Debug, Clone)]
pub struct ChangeMembershipResponse {
    pub message: String,
    pub success: bool,
    pub leader_id: Option<NodeId>,
}

#[derive(Debug, Clone)]
pub struct WriteRequest {
    pub command: String,
}

#[derive(Debug, Clone)]
pub struct WriteResponse {
    pub message: String,
    pub success: bool,
    pub leader_id: Option<NodeId>,
}

#[derive(Debug, Clone)]
pub struct ReadRequest {
    pub command: String,
}

#[derive(Debug, Clone)]
pub struct ReadResponse {
    pub message: String,
    pub success: bool,
    pub leader_id: Option<NodeId>,
}
