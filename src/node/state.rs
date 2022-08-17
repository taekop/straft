use anyhow::Result;
use std::{
    collections::{HashMap, HashSet},
    iter,
    ops::RangeFrom,
    sync::mpsc::SyncSender,
};

use crate::{Command, Entry, NodeId};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Role {
    FOLLOWER,
    CANDIDATE,
    LEADER,
    SHUTDOWN,
}

pub struct NodeState {
    // role
    pub role: Role,
    // persistent
    pub current_term: u64,
    pub voted_for: Option<NodeId>,
    log: Vec<Entry>,
    // volatile
    pub commit_index: usize,
    pub last_applied: usize,
    // for candidate
    pub candidate_term: u64,
    pub votes: HashSet<NodeId>,
    // for leader
    pub next_index: HashMap<NodeId, usize>,
    pub match_index: HashMap<NodeId, usize>,
    // extra
    pub leader_id: Option<NodeId>,
    write_responser: Vec<Option<SyncSender<Result<String>>>>,
    // joint consensus
    joint_consensus: bool,
    new_members: HashSet<NodeId>,
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
                command: Command::Empty,
            }],
            commit_index: 0,
            last_applied: 0,
            candidate_term: 0,
            votes: HashSet::new(),
            next_index: HashMap::new(),
            match_index: HashMap::new(),
            leader_id: None,
            write_responser: vec![None],
            joint_consensus: false,
            new_members: HashSet::new(),
        }
    }

    pub fn initialize_candidate_state(&mut self, id: NodeId) {
        self.candidate_term = self.current_term;
        self.votes = HashSet::from_iter(iter::once(id));
    }

    pub fn initialize_leader_state(&mut self, id: NodeId) {
        self.leader_id = Some(id);
        self.next_index = HashMap::new();
        self.match_index = HashMap::new();
    }

    pub fn is_role(&self, _role: Role) -> bool {
        self.role == _role
    }

    pub fn change_role(&mut self, _role: Role) {
        self.role = _role;
    }

    pub fn push_log(&mut self, entry: Entry, sender: Option<SyncSender<Result<String>>>) {
        if let Command::ChangeMembership(members) = &entry.command {
            self.start_joint_consensus(members.clone())
        }
        self.log.push(entry);
        self.write_responser.push(sender);
    }

    pub fn splice_log(&mut self, from: usize, entries: Vec<Entry>) {
        for entry in entries.iter() {
            if let Command::ChangeMembership(members) = &entry.command {
                self.start_joint_consensus(members.clone())
            }   
        }
        self.write_responser
            .splice(from.., vec![None; entries.len()]);
        self.log.splice(from.., entries);
    }

    pub fn log(&self, ind: usize) -> &Entry {
        &self.log[ind]
    }

    pub fn log_range_from(&self, range: RangeFrom<usize>) -> &[Entry] {
        &self.log[range]
    }

    pub fn last_log(&self) -> &Entry {
        self.log.last().unwrap()
    }

    pub fn write_responser(&self, ind: usize) -> &Option<SyncSender<Result<String>>> {
        &self.write_responser[ind]
    }

    pub fn is_joint_consensus(&self) -> bool {
        self.joint_consensus
    }

    pub fn start_joint_consensus(&mut self, new_members: HashSet<NodeId>) {
        self.joint_consensus = true;
        self.new_members = new_members;
    }

    pub fn finish_joint_consensus(&mut self) {
        self.joint_consensus = false;
        self.new_members = HashSet::new();
    }

    pub fn new_members(&self) -> &HashSet<NodeId> {
        &self.new_members
    }
}
