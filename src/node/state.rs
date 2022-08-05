use std::collections::HashMap;

use crate::node::role::Role;
use crate::{Command, Entry, NodeId};

pub struct NodeState<C: Command> {
    // role
    role: Role,
    // persistent
    current_term: u64,
    voted_for: Option<NodeId>,
    log: Vec<Entry<C>>,
    // volatile
    commit_index: u64,
    last_applied: u64,
    next_index: HashMap<NodeId, u64>,
    match_index: HashMap<NodeId, u64>,
    // extra
    leader_id: Option<NodeId>,
}

impl<C: Command> NodeState<C> {
    pub fn new() -> Self {
        NodeState {
            role: Role::FOLLOWER,
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            next_index: HashMap::new(),
            match_index: HashMap::new(),
            leader_id: None,
        }
    }

    pub fn is_follower(&self) -> bool {
        match &self.role {
            Role::FOLLOWER => true,
            _ => false,
        }
    }

    pub fn is_candidate(&self) -> bool {
        match &self.role {
            Role::CANDIDATE => true,
            _ => false,
        }
    }

    pub fn is_leader(&self) -> bool {
        match &self.role {
            Role::LEADER => true,
            _ => false,
        }
    }
}
