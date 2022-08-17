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
    NONVOTER,
    SHUTDOWN,
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
            id,
            members,
            new_members: HashSet::new(),
            non_voting_members: HashSet::new(),
            joint_consensus: false,
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

    pub fn log(&self, ind: usize) -> &Entry {
        &self.log[ind]
    }

    pub fn log_range_from(&self, range: RangeFrom<usize>) -> &[Entry] {
        &self.log[range]
    }

    pub fn last_log(&self) -> &Entry {
        self.log.last().unwrap()
    }

    pub fn push_log(&mut self, entry: Entry, sender: Option<SyncSender<Result<String>>>) {
        self.change_membership(&entry.command);
        self.log.push(entry);
        self.write_responser.push(sender);
    }

    pub fn splice_log(&mut self, from: usize, entries: Vec<Entry>) {
        for entry in entries.iter() {
            self.change_membership(&entry.command);
        }
        self.write_responser
            .splice(from.., vec![None; entries.len()]);
        self.log.splice(from.., entries);
    }

    pub fn write_responser(&self, ind: usize) -> &Option<SyncSender<Result<String>>> {
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
                        self.last_log().index
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
}
