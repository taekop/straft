use crate::types::MyCommand;

tonic::include_proto!("raft");

impl Into<straft::Entry<MyCommand>> for Entry {
    fn into(self) -> straft::Entry<MyCommand> {
        straft::Entry {
            index: self.index.unwrap(),
            term: self.term.unwrap(),
            command: MyCommand(self.command.unwrap()),
        }
    }
}

impl From<straft::Entry<MyCommand>> for Entry {
    fn from(entry: straft::Entry<MyCommand>) -> Entry {
        Entry {
            index: Some(entry.index),
            term: Some(entry.term),
            command: Some(entry.command.0),
        }
    }
}

impl Into<straft::rpc::AppendEntriesRequest<MyCommand>> for AppendEntriesRequest {
    fn into(self) -> straft::rpc::AppendEntriesRequest<MyCommand> {
        straft::rpc::AppendEntriesRequest {
            term: self.term.unwrap(),
            leader_id: self.leader_id.unwrap(),
            prev_log_index: self.prev_log_index.unwrap(),
            prev_log_term: self.prev_log_term.unwrap(),
            entries: self.entries.into_iter().map(|x| x.into()).collect(),
            leader_commit: self.leader_commit.unwrap(),
        }
    }
}

impl From<straft::rpc::AppendEntriesRequest<MyCommand>> for AppendEntriesRequest {
    fn from(req: straft::rpc::AppendEntriesRequest<MyCommand>) -> AppendEntriesRequest {
        AppendEntriesRequest {
            term: Some(req.term),
            leader_id: Some(req.leader_id),
            prev_log_index: Some(req.prev_log_index),
            prev_log_term: Some(req.prev_log_term),
            entries: req.entries.into_iter().map(|x| Entry::from(x)).collect(),
            leader_commit: Some(req.leader_commit),
        }
    }
}

impl Into<straft::rpc::AppendEntriesResponse> for AppendEntriesResponse {
    fn into(self) -> straft::rpc::AppendEntriesResponse {
        straft::rpc::AppendEntriesResponse {
            term: self.term.unwrap(),
            success: self.success.unwrap(),
        }
    }
}

impl From<straft::rpc::AppendEntriesResponse> for AppendEntriesResponse {
    fn from(res: straft::rpc::AppendEntriesResponse) -> AppendEntriesResponse {
        AppendEntriesResponse {
            term: Some(res.term),
            success: Some(res.success),
        }
    }
}

impl Into<straft::rpc::RequestVoteRequest> for RequestVoteRequest {
    fn into(self) -> straft::rpc::RequestVoteRequest {
        straft::rpc::RequestVoteRequest {
            term: self.term.unwrap(),
            candidate_id: self.candidate_id.unwrap(),
            last_log_index: self.last_log_index.unwrap(),
            last_log_term: self.last_log_term.unwrap(),
        }
    }
}

impl From<straft::rpc::RequestVoteRequest> for RequestVoteRequest {
    fn from(req: straft::rpc::RequestVoteRequest) -> RequestVoteRequest {
        RequestVoteRequest {
            term: Some(req.term),
            candidate_id: Some(req.candidate_id),
            last_log_index: Some(req.last_log_index),
            last_log_term: Some(req.last_log_term),
        }
    }
}

impl Into<straft::rpc::RequestVoteResponse> for RequestVoteResponse {
    fn into(self) -> straft::rpc::RequestVoteResponse {
        straft::rpc::RequestVoteResponse {
            term: self.term.unwrap(),
            vote_granted: self.vote_granted.unwrap(),
        }
    }
}

impl From<straft::rpc::RequestVoteResponse> for RequestVoteResponse {
    fn from(res: straft::rpc::RequestVoteResponse) -> RequestVoteResponse {
        RequestVoteResponse {
            term: Some(res.term),
            vote_granted: Some(res.vote_granted),
        }
    }
}

impl Into<straft::rpc::AppendLogRequest<MyCommand>> for AppendLogRequest {
    fn into(self) -> straft::rpc::AppendLogRequest<MyCommand> {
        straft::rpc::AppendLogRequest {
            command: MyCommand(self.command.unwrap()),
        }
    }
}

impl From<straft::rpc::AppendLogRequest<MyCommand>> for AppendLogRequest {
    fn from(req: straft::rpc::AppendLogRequest<MyCommand>) -> AppendLogRequest {
        AppendLogRequest {
            command: Some(req.command.0),
        }
    }
}

impl Into<straft::rpc::AppendLogResponse> for AppendLogResponse {
    fn into(self) -> straft::rpc::AppendLogResponse {
        straft::rpc::AppendLogResponse {
            success: self.success.unwrap(),
            leader_id: self.leader_id,
            leader_address: self.leader_address,
        }
    }
}

impl From<straft::rpc::AppendLogResponse> for AppendLogResponse {
    fn from(res: straft::rpc::AppendLogResponse) -> AppendLogResponse {
        AppendLogResponse {
            success: Some(res.success),
            leader_id: res.leader_id,
            leader_address: res.leader_address,
        }
    }
}
