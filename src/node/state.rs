use anyhow::Result;
use std::{
    collections::{HashMap, HashSet},
    sync::mpsc::SyncSender,
};

use crate::{Command, Entry, NodeId};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Role {
    FOLLOWER,
    CANDIDATE,
    LEADER,
    NONVOTER,
    SHUTDOWN,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = match self {
            Role::FOLLOWER => "\x1b[37m",
            Role::CANDIDATE => "\x1b[33m",
            Role::LEADER => "\x1b[36m",
            Role::NONVOTER => "\x1b[30m",
            Role::SHUTDOWN => "\x1b[31m",
        };
        write!(f, "{color}{:?}\x1b[0m", self)
    }
}

pub struct NodeState {
    // role
    role: Role,
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
    // membership
    id: NodeId,
    members: HashSet<NodeId>,
    new_members: HashSet<NodeId>,
    non_voting_members: HashSet<NodeId>,
    joint_consensus: bool,
    // snapshot
    last_included_index: usize,
    last_included_term: u64,
}

impl NodeState {
    pub fn new(id: NodeId, members: HashSet<NodeId>) -> Self {
        let role = if members.contains(&id) {
            Role::FOLLOWER
        } else {
            Role::NONVOTER
        };
        NodeState {
            role,
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            candidate_term: 0,
            votes: HashSet::new(),
            next_index: HashMap::new(),
            match_index: HashMap::new(),
            leader_id: None,
            write_responser: Vec::new(),
            id,
            members,
            new_members: HashSet::new(),
            non_voting_members: HashSet::new(),
            joint_consensus: false,
            last_included_index: 0,
            last_included_term: 0,
        }
    }

    pub fn initialize_candidate_state(&mut self, id: NodeId) {
        self.candidate_term = self.current_term;
        self.votes = HashSet::from_iter(std::iter::once(id));
    }

    pub fn initialize_leader_state(&mut self, id: NodeId) {
        self.leader_id = Some(id);
        self.next_index = HashMap::new();
        self.match_index = HashMap::new();
    }

    pub fn role(&self) -> Role {
        self.role
    }

    pub fn is_role(&self, _role: Role) -> bool {
        self.role == _role
    }

    pub fn change_role(&mut self, _role: Role) {
        self.role = _role;
    }

    pub fn log(&self, ind: usize) -> &Entry {
        &self.log[ind - self.last_included_index - 1]
    }

    pub fn log_range_from(&self, start: usize) -> &[Entry] {
        let start = start - self.last_included_index - 1;
        self.log.get(start..).unwrap_or_default()
    }

    pub fn log_info(&self, ind: usize) -> (usize, u64) {
        if ind == self.last_included_index {
            (self.last_included_index, self.last_included_term)
        } else {
            let ind = ind - self.last_included_index - 1;
            let entry = &self.log[ind];
            (entry.index, entry.term)
        }
    }

    pub fn last_log_info(&self) -> (usize, u64) {
        if self.log.is_empty() {
            (self.last_included_index, self.last_included_term)
        } else {
            let entry = &self.log.last().unwrap();
            (entry.index, entry.term)
        }
    }

    pub fn push_log(&mut self, entry: Entry, sender: Option<SyncSender<Result<String>>>) {
        self.change_membership(&entry.command);
        self.log.push(entry);
        self.write_responser.push(sender);
    }

    pub fn splice_log(&mut self, from: usize, entries: Vec<Entry>) {
        let from = from - self.last_included_index - 1;
        for entry in entries.iter() {
            self.change_membership(&entry.command);
        }
        self.write_responser
            .splice(from.., vec![None; entries.len()]);
        self.log.splice(from.., entries);
    }

    pub fn write_responser(&self, ind: usize) -> &Option<SyncSender<Result<String>>> {
        let ind = ind - self.last_included_index - 1;
        &self.write_responser[ind]
    }

    pub fn is_joint_consensus(&self) -> bool {
        self.joint_consensus
    }

    pub fn members(&self) -> &HashSet<NodeId> {
        &self.members
    }

    pub fn all_members(&self) -> HashSet<NodeId> {
        HashSet::from_iter(
            HashSet::from_iter(self.members.union(&self.new_members).into_iter().cloned())
                .union(&self.non_voting_members)
                .into_iter()
                .cloned(),
        )
    }

    fn change_membership(&mut self, command: &Command) {
        if let Command::ChangeMembership(new_members, non_voting_members) = command {
            if let Some(new_members) = new_members {
                self.start_joint_consensus(new_members.clone());
            } else if let Some(non_voting_members) = non_voting_members {
                self.non_voting_members = non_voting_members.clone();
            }
        }
    }

    fn start_joint_consensus(&mut self, new_members: HashSet<NodeId>) {
        self.joint_consensus = true;
        self.new_members = new_members;
    }

    pub fn finish_joint_consensus(&mut self) {
        self.joint_consensus = false;
        self.members = HashSet::new();
        std::mem::swap(&mut self.members, &mut self.new_members);
    }

    pub fn in_cluster(&self) -> bool {
        self.members.contains(&self.id) || self.new_members.contains(&self.id)
    }

    pub fn in_non_voting_members(&self) -> bool {
        self.non_voting_members.contains(&self.id)
    }

    pub fn is_majority(&self, ids: &HashSet<NodeId>) -> bool {
        let _is_majority = |members: &HashSet<NodeId>| {
            let majority = members.iter().count() / 2 + 1;
            let count = members.intersection(ids).count();
            count >= majority
        };
        _is_majority(&self.members) && (!self.joint_consensus || _is_majority(&self.new_members))
    }

    pub fn majority_match_index(&self) -> usize {
        let _majority_match_index = |members: &HashSet<NodeId>| {
            let majority = members.iter().count() / 2 + 1;
            let mut match_indices: Vec<usize> = members
                .iter()
                .map(|id| {
                    if id == &self.id {
                        self.last_log_info().0
                    } else {
                        *self.match_index.get(id).unwrap_or(&0)
                    }
                })
                .collect();
            match_indices.sort();
            match_indices.get(majority).unwrap_or(&0).clone()
        };
        let mut res = _majority_match_index(&self.members);
        if self.is_joint_consensus() {
            res = std::cmp::min(res, _majority_match_index(&self.new_members));
        }
        res
    }

    pub fn in_snapshot(&self, ind: usize) -> bool {
        ind <= self.last_included_index
    }

    pub fn should_save_snapshot(&self, threshold: usize) -> bool {
        self.last_applied > self.last_included_index + threshold
    }

    pub fn save_snapshot(&mut self, last_included_index: usize, last_included_term: u64) {
        let new_last_included_index = last_included_index - self.last_included_index - 1;
        self.log.drain(..=new_last_included_index);
        self.last_included_index = last_included_index;
        self.last_included_term = last_included_term;
    }

    pub fn install_snapshot(&mut self, last_included_index: usize, last_included_term: u64) {
        self.commit_index = last_included_index;
        self.last_applied = last_included_index;
        self.last_included_index = last_included_index;
        self.last_included_term = last_included_term;
    }
}
