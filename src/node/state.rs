use std::collections::HashMap;

use crate::{Entry, NodeId};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Role {
    FOLLOWER,
    CANDIDATE,
    LEADER,
}

pub struct NodeState {
    // role
    pub role: Role,
    // persistent
    pub current_term: u64,
    pub voted_for: Option<NodeId>,
    pub log: Vec<Entry>,
    // volatile
    pub commit_index: usize,
    pub last_applied: usize,
    // for candidate
    pub candidate_term: u64,
    pub vote_cnt: u64,
    // for leader
    pub next_index: HashMap<NodeId, usize>,
    pub match_index: HashMap<NodeId, usize>,
    // extra
    pub leader_id: Option<NodeId>,
    pub leader_address: Option<String>,
}

impl NodeState {
    pub fn new() -> Self {
        NodeState {
            role: Role::FOLLOWER,
            current_term: 0,
            voted_for: None,
            log: vec![Entry {
                index: 0,
                term: 0,
                command: String::default(),
                sender: None,
            }],
            commit_index: 0,
            last_applied: 0,
            candidate_term: 0,
            vote_cnt: 0,
            next_index: HashMap::new(),
            match_index: HashMap::new(),
            leader_id: None,
            leader_address: None,
        }
    }

    pub fn initialize_candidate_state(&mut self) {
        self.candidate_term = self.current_term;
        self.vote_cnt = 1;
    }

    pub fn initialize_leader_state(&mut self, id: NodeId, address: String) {
        self.leader_id = Some(id);
        self.leader_address = Some(address);
        self.next_index = HashMap::new();
        self.match_index = HashMap::new();
    }

    pub fn is_role(&self, _role: Role) -> bool {
        self.role == _role
    }

    pub fn change_role(&mut self, _role: Role) {
        self.role = _role;
    }

    pub fn last_log(&self) -> &Entry {
        self.log.last().unwrap()
    }
}
