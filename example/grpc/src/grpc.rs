// gRPC module
// Implement conversions between RPC and gRPC

use std::collections::HashSet;

tonic::include_proto!("raft");

impl Into<straft::Entry> for Entry {
    fn into(self) -> straft::Entry {
        let command: straft::Command = match self.command.unwrap() {
            entry::Command::Empty(Empty {}) => straft::Command::Empty,
            entry::Command::Write(command) => straft::Command::Write(command),
            entry::Command::ChangeMembership(ChangeMembership { members }) => {
                straft::Command::ChangeMembership(HashSet::from_iter(members.into_iter()))
            }
        };
        straft::Entry {
            index: self.index.unwrap() as usize,
            term: self.term.unwrap(),
            command,
        }
    }
}

impl From<straft::Entry> for Entry {
    fn from(entry: straft::Entry) -> Entry {
        let command = Some(match entry.command {
            straft::Command::Empty => entry::Command::Empty(Empty {}),
            straft::Command::Write(command) => entry::Command::Write(command),
            straft::Command::ChangeMembership(members) => {
                entry::Command::ChangeMembership(ChangeMembership {
                    members: Vec::from_iter(members),
                })
            }
        });
        Entry {
            index: Some(entry.index as u64),
            term: Some(entry.term),
            command,
        }
    }
}

impl Into<straft::rpc::AppendEntriesRequest> for AppendEntriesRequest {
    fn into(self) -> straft::rpc::AppendEntriesRequest {
        straft::rpc::AppendEntriesRequest {
            term: self.term.unwrap(),
            leader_id: self.leader_id.unwrap(),
            prev_log_index: self.prev_log_index.unwrap() as usize,
            prev_log_term: self.prev_log_term.unwrap(),
            entries: self.entries.into_iter().map(|x| x.into()).collect(),
            leader_commit: self.leader_commit.unwrap() as usize,
        }
    }
}

impl From<straft::rpc::AppendEntriesRequest> for AppendEntriesRequest {
    fn from(req: straft::rpc::AppendEntriesRequest) -> AppendEntriesRequest {
        AppendEntriesRequest {
            term: Some(req.term),
            leader_id: Some(req.leader_id),
            prev_log_index: Some(req.prev_log_index as u64),
            prev_log_term: Some(req.prev_log_term),
            entries: req.entries.into_iter().map(|x| Entry::from(x)).collect(),
            leader_commit: Some(req.leader_commit as u64),
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
            last_log_index: self.last_log_index.unwrap() as usize,
            last_log_term: self.last_log_term.unwrap(),
        }
    }
}

impl From<straft::rpc::RequestVoteRequest> for RequestVoteRequest {
    fn from(req: straft::rpc::RequestVoteRequest) -> RequestVoteRequest {
        RequestVoteRequest {
            term: Some(req.term),
            candidate_id: Some(req.candidate_id),
            last_log_index: Some(req.last_log_index as u64),
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

impl Into<straft::rpc::ChangeMembershipRequest> for ChangeMembershipRequest {
    fn into(self) -> straft::rpc::ChangeMembershipRequest {
        straft::rpc::ChangeMembershipRequest {
            members: HashSet::from_iter(self.members.into_iter()),
        }
    }
}

impl From<straft::rpc::ChangeMembershipRequest> for ChangeMembershipRequest {
    fn from(req: straft::rpc::ChangeMembershipRequest) -> ChangeMembershipRequest {
        ChangeMembershipRequest {
            members: req.members.into_iter().collect(),
        }
    }
}

impl Into<straft::rpc::ChangeMembershipResponse> for ChangeMembershipResponse {
    fn into(self) -> straft::rpc::ChangeMembershipResponse {
        straft::rpc::ChangeMembershipResponse {
            message: self.message.unwrap(),
            success: self.success.unwrap(),
            leader_id: self.leader_id,
        }
    }
}

impl From<straft::rpc::ChangeMembershipResponse> for ChangeMembershipResponse {
    fn from(res: straft::rpc::ChangeMembershipResponse) -> ChangeMembershipResponse {
        ChangeMembershipResponse {
            message: Some(res.message),
            success: Some(res.success),
            leader_id: res.leader_id,
        }
    }
}

impl Into<straft::rpc::WriteRequest> for WriteRequest {
    fn into(self) -> straft::rpc::WriteRequest {
        straft::rpc::WriteRequest {
            command: self.command.unwrap(),
        }
    }
}

impl From<straft::rpc::WriteRequest> for WriteRequest {
    fn from(req: straft::rpc::WriteRequest) -> WriteRequest {
        WriteRequest {
            command: Some(req.command),
        }
    }
}

impl Into<straft::rpc::WriteResponse> for WriteResponse {
    fn into(self) -> straft::rpc::WriteResponse {
        straft::rpc::WriteResponse {
            message: self.message.unwrap(),
            success: self.success.unwrap(),
            leader_id: self.leader_id,
        }
    }
}

impl From<straft::rpc::WriteResponse> for WriteResponse {
    fn from(res: straft::rpc::WriteResponse) -> WriteResponse {
        WriteResponse {
            message: Some(res.message),
            success: Some(res.success),
            leader_id: res.leader_id,
        }
    }
}

impl Into<straft::rpc::ReadRequest> for ReadRequest {
    fn into(self) -> straft::rpc::ReadRequest {
        straft::rpc::ReadRequest {
            command: self.command.unwrap(),
        }
    }
}

impl From<straft::rpc::ReadRequest> for ReadRequest {
    fn from(req: straft::rpc::ReadRequest) -> ReadRequest {
        ReadRequest {
            command: Some(req.command),
        }
    }
}

impl Into<straft::rpc::ReadResponse> for ReadResponse {
    fn into(self) -> straft::rpc::ReadResponse {
        straft::rpc::ReadResponse {
            message: self.message.unwrap(),
            success: self.success.unwrap(),
            leader_id: self.leader_id,
        }
    }
}

impl From<straft::rpc::ReadResponse> for ReadResponse {
    fn from(res: straft::rpc::ReadResponse) -> ReadResponse {
        ReadResponse {
            message: Some(res.message),
            success: Some(res.success),
            leader_id: res.leader_id,
        }
    }
}
