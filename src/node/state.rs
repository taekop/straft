use crate::{Command, Entry, NodeId};
use futures::executor::block_on;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy)]
pub enum Role {
    FOLLOWER,
    CANDIDATE,
    LEADER,
}

pub struct NodeState<C: Command> {
    // role
    pub role: Arc<Mutex<Role>>,
    // persistent
    pub current_term: Arc<Mutex<u64>>,
    pub voted_for: Arc<Mutex<Option<NodeId>>>,
    pub log: Vec<Entry<C>>,
    // volatile
    pub commit_index: u64,
    pub last_applied: u64,
    pub next_index: Arc<Mutex<HashMap<NodeId, u64>>>,
    pub match_index: Arc<Mutex<HashMap<NodeId, u64>>>,
    // extra
    pub leader_id: Arc<Mutex<Option<NodeId>>>,
}

impl<C: Command> NodeState<C> {
    pub fn new() -> Self {
        NodeState {
            role: Arc::new(Mutex::new(Role::FOLLOWER)),
            current_term: Arc::new(Mutex::new(0)),
            voted_for: Arc::new(Mutex::new(None)),
            log: vec![Entry {
                index: 0,
                term: 0,
                command: C::default(),
            }],
            commit_index: 0,
            last_applied: 0,
            next_index: Arc::new(Mutex::new(HashMap::new())),
            match_index: Arc::new(Mutex::new(HashMap::new())),
            leader_id: Arc::new(Mutex::new(None)),
        }
    }

    pub fn initialize_leader_state(&self, id: NodeId) {
        let mut leader_id = block_on(self.leader_id.lock());
        let mut next_index = block_on(self.next_index.lock());
        let mut match_index = block_on(self.match_index.lock());
        *leader_id = Some(id);
        *next_index = HashMap::new();
        *match_index = HashMap::new();
    }

    pub fn is_role(&self, _role: Role) -> bool {
        let role = self.role.lock();
        matches!(role, _role)
    }

    pub fn change_role(&self, _role: Role) {
        let mut role = block_on(self.role.lock());
        *role = _role;
    }
}
