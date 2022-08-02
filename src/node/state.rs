use std::collections::HashMap;

use crate::NodeId;
use crate::node::role::Role;
use crate::node::entry::Entry;

pub struct NodeState {
    // role
    role: Role,
    // persistent
    current_term: u64,
    voted_for: Option<NodeId>,
    log: Vec<Entry>,
    // volatile
    commit_index: u64,
    last_applied: u64,
    next_index: HashMap<NodeId, u64>,
    match_index: HashMap<NodeId, u64>,
    // extra
    leader_id: Option<NodeId>,
}

impl NodeState {
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
}
