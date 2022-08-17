use std::sync::mpsc;

pub mod actor;
pub mod client;
mod election_timer;
pub mod logger;
mod state;

use self::{
    actor::{Request, RequestMessage},
    client::{ExternalNodeClient, InternalNodeClient},
    election_timer::ElectionTimer,
    logger::Logger,
    state::{NodeState, Role},
};

use crate::{
    rpc::{
        AppendEntriesRequest, AppendEntriesResponse, ChangeMembershipRequest,
        ChangeMembershipResponse, ReadRequest, ReadResponse, RequestVoteRequest,
        RequestVoteResponse, WriteRequest, WriteResponse,
    },
    state_machine::StateMachineClient,
    ClusterConfig, Command, Entry, NodeId, ResponseMessage,
};

pub struct Node<SM: StateMachineClient, Client: ExternalNodeClient> {
    id: NodeId,
    config: ClusterConfig,
    state_machine: SM,
    logger: Logger,
    // client to send request to other nodes
    external_client: Client,
    // client to send request to self
    internal_client: InternalNodeClient,
    // receive requests
    receiver: mpsc::Receiver<Request>,
    // raft state, not state machine state
    state: NodeState,
    // about election time
    election_timer: ElectionTimer,
    // shutdown actor if false
    running: bool,
}

impl<SM: StateMachineClient, Client: ExternalNodeClient> Node<SM, Client> {
    fn handle_append_entries(&mut self, req: AppendEntriesRequest) -> AppendEntriesResponse {
        self.election_timer.reset(true);
        if req.term > self.state.current_term
            && (self.state.is_role(Role::CANDIDATE) || self.state.is_role(Role::LEADER))
        {
            self.change_role(Role::FOLLOWER);
        }

        let valid_term = req.term >= self.state.current_term;
        let valid_log_term = self.state.last_log().term == req.prev_log_term;
        if valid_term && valid_log_term {
            self.state.current_term = req.term;
            self.state.leader_id = Some(req.leader_id);

            self.state.splice_log(req.prev_log_index + 1, req.entries);
            if self.state.is_role(Role::NONVOTER) && self.state.in_cluster() {
                self.change_role(Role::FOLLOWER);
            }

            if req.leader_commit > self.state.commit_index {
                self.state.commit_index =
                    std::cmp::min(req.leader_commit, self.state.last_log().index);
            }

            AppendEntriesResponse {
                term: self.state.current_term,
                success: true,
            }
        } else {
            AppendEntriesResponse {
                term: self.state.current_term,
                success: false,
            }
        }
    }

    fn handle_request_vote(&mut self, req: RequestVoteRequest) -> RequestVoteResponse {
        self.election_timer.reset(false);
        let current_term = self.state.current_term;
        let voted_for = self.state.voted_for.clone();
        let last_log = self.state.last_log();

        let valid_role = !self.state.is_role(Role::NONVOTER) && !self.state.is_role(Role::SHUTDOWN);
        let valid_time = self.election_timer.is_current_leader_timeout();
        let valid_term = req.term >= current_term;
        let valid_vote = req.term != current_term
            || voted_for.is_none()
            || voted_for.unwrap() == req.candidate_id;
        let valid_log = req.last_log_index >= last_log.index && req.last_log_term >= last_log.term;
        if valid_role && valid_time && valid_term && valid_vote && valid_log {
            self.state.current_term = req.term;
            self.state.voted_for = Some(req.candidate_id);
            RequestVoteResponse {
                term: self.state.current_term,
                vote_granted: true,
            }
        } else {
            RequestVoteResponse {
                term: self.state.current_term,
                vote_granted: false,
            }
        }
    }

    fn handle_change_membership(
        &mut self,
        req: ChangeMembershipRequest,
    ) -> ChangeMembershipResponse {
        if self.state.is_role(Role::LEADER) {
            if self.state.is_joint_consensus() {
                ChangeMembershipResponse {
                    message: format!("Retry after current membership change finished"),
                    success: true,
                    leader_id: None,
                }
            } else {
                let last_log = self.state.last_log();
                let new_entry = Entry {
                    index: last_log.index + 1,
                    term: self.state.current_term,
                    command: Command::ChangeMembership(req.new_members, req.non_voting_members),
                };

                self.state.push_log(new_entry, None);
                if self.state.is_role(Role::NONVOTER) && self.state.in_cluster() {
                    self.change_role(Role::FOLLOWER);
                }

                ChangeMembershipResponse {
                    message: format!(""),
                    success: true,
                    leader_id: None,
                }
            }
        } else {
            ChangeMembershipResponse {
                message: format!("Retry to leader"),
                success: false,
                leader_id: self.state.leader_id.clone(),
            }
        }
    }

    fn handle_write(&mut self, req: WriteRequest) -> mpsc::Receiver<ResponseMessage> {
        let (tx, rx) = mpsc::sync_channel(1);
        if self.state.is_role(Role::LEADER) {
            let (tx2, rx2) = mpsc::sync_channel(1);
            let last_log = self.state.last_log();
            let new_entry = Entry {
                index: last_log.index + 1,
                term: self.state.current_term,
                command: Command::Write(req.command),
            };
            self.state.push_log(new_entry, Some(tx2));
            std::thread::spawn(move || {
                let (message, success) = match rx2.recv() {
                    Ok(Ok(message)) => (message, true),
                    Ok(Err(message)) => (message.to_string(), false),
                    Err(message) => (message.to_string(), false),
                };

                tx.send(ResponseMessage::Write(WriteResponse {
                    message,
                    success,
                    leader_id: None,
                }))
                .ok();
            });
        } else {
            tx.send(ResponseMessage::Write(WriteResponse {
                message: format!("Retry to leader"),
                success: false,
                leader_id: self.state.leader_id.clone(),
            }))
            .ok();
        }
        rx
    }

    fn handle_read(&mut self, req: ReadRequest) -> mpsc::Receiver<ResponseMessage> {
        let (tx, rx) = mpsc::sync_channel(1);
        let state_machine = self.state_machine.clone();
        std::thread::spawn(move || {
            let res = state_machine.read(req.command);
            let (message, success) = match res {
                Ok(message) => (message, true),
                Err(message) => (message.to_string(), false),
            };
            tx.send(ResponseMessage::Read(ReadResponse {
                message,
                success,
                leader_id: None,
            }))
            .ok();
        });
        rx
    }

    fn handle_heartbeat(&mut self) -> ResponseMessage {
        if self.state.in_cluster() {
            if self.election_timer.is_timeout()
                && (self.state.is_role(Role::FOLLOWER) || self.state.is_role(Role::CANDIDATE))
            {
                self.change_role(Role::CANDIDATE);
            } else if self.state.is_role(Role::LEADER) {
                self.request_append_entries();
            }
        }

        if self.state.commit_index > self.state.last_applied {
            for i in self.state.last_applied + 1..=self.state.commit_index {
                self.execute(i);
            }
            self.state.last_applied = self.state.commit_index;
        }
        ResponseMessage::Heartbeat
    }

    fn handle_append_entries_result(
        &mut self,
        leader_term: u64,
        follower_id: NodeId,
        last_log_index: usize,
        req: AppendEntriesResponse,
    ) -> ResponseMessage {
        if self.state.current_term == leader_term && self.state.is_role(Role::LEADER) {
            if req.term > leader_term {
                self.state.current_term = req.term;
                self.change_role(Role::FOLLOWER);
            } else if last_log_index > 0 {
                if req.success {
                    self.state
                        .match_index
                        .insert(follower_id.clone(), last_log_index);
                    self.state
                        .next_index
                        .insert(follower_id.clone(), last_log_index + 1);
                } else {
                    self.state.next_index.entry(follower_id).and_modify(|e| {
                        *e -= 1;
                    });
                }
                let majority_match_index = self.state.majority_match_index();
                if majority_match_index > self.state.commit_index {
                    self.state.commit_index = majority_match_index;
                }
            }
        }
        ResponseMessage::AppendEntriesResult
    }

    fn handle_request_vote_result(
        &mut self,
        follower_id: NodeId,
        candidate_term: u64,
        req: RequestVoteResponse,
    ) -> ResponseMessage {
        let current_term = self.state.current_term;
        if current_term == candidate_term && self.state.is_role(Role::CANDIDATE) {
            if req.term > current_term {
                self.state.current_term = req.term;
                self.change_role(Role::FOLLOWER);
            } else if req.vote_granted {
                self.state.votes.insert(follower_id);
                if self.state.is_majority(&self.state.votes) {
                    self.change_role(Role::LEADER);
                }
            }
        }
        ResponseMessage::RequestVoteResult
    }

    fn execute(&mut self, ind: usize) {
        let command = self.state.log(ind).command.clone();
        self.log_debug(format!("Execute: {:?}", command));
        if let Command::Write(command) = command {
            let res = self.state_machine.write(command);
            let tx = self.state.write_responser(ind).clone();
            if let Some(sender) = tx {
                sender.send(res).ok();
            }
        } else if let Command::ChangeMembership(new_members, _) = command {
            if new_members.is_some() {
                self.finish_joint_consensus();
            }
        }
    }

    fn change_role(&mut self, role: Role) {
        self.log_info(format!("Change role to {:?}", role));
        self.state.change_role(role);
        match role {
            Role::CANDIDATE => {
                self.state.initialize_candidate_state(self.id.clone());
                self.state.leader_id = None;
                self.run_for_leader();
            }
            Role::LEADER => {
                self.state.initialize_leader_state(self.id.clone());
                self.request_append_entries();
            }
            Role::SHUTDOWN => {
                self.shutdown();
            }
            _ => {}
        }
    }

    fn run_for_leader(&mut self) {
        let candidate_term = self.state.current_term + 1;
        self.log_info(format!("Run for leader in term {:?}", candidate_term));
        self.election_timer.reset(false);
        self.state.current_term = candidate_term;
        self.state.voted_for = Some(self.id.clone());

        let last_log = self.state.last_log();
        let msg = RequestVoteRequest {
            term: candidate_term,
            candidate_id: self.id.clone(),
            last_log_index: last_log.index,
            last_log_term: last_log.term,
        };
        let members = self.state.members();
        for follower_id in members {
            if follower_id == &self.id {
                continue;
            }
            let msg = msg.clone();
            let follower_id = follower_id.clone();
            let mut external_client = self.external_client.clone();
            let internal_client = self.internal_client.clone();
            std::thread::spawn(move || {
                let res = external_client.request_vote(follower_id.clone(), msg);
                if let Ok(res) = res {
                    internal_client.send(RequestMessage::RequestVoteResult(
                        candidate_term,
                        follower_id,
                        res,
                    ));
                }
            });
        }
    }

    fn request_append_entries(&mut self) {
        let leader_term = self.state.current_term;
        let last_log_index = self.state.last_log().index;

        let members = self.state.all_members();
        for follower_id in members.iter() {
            if follower_id == &self.id {
                continue;
            }
            let next_index = *self
                .state
                .next_index
                .entry(follower_id.clone())
                .or_insert(last_log_index + 1);
            let prev_log_index = next_index - 1;
            let prev_log_term = self.state.log(prev_log_index).term;
            let entries = self.state.log_range_from(next_index..).to_vec();
            let msg = AppendEntriesRequest {
                term: self.state.current_term,
                leader_id: self.id.clone(),
                prev_log_index,
                prev_log_term,
                entries,
                leader_commit: self.state.commit_index,
            };
            let follower_id = follower_id.clone();
            let mut external_client = self.external_client.clone();
            let internal_client = self.internal_client.clone();
            std::thread::spawn(move || {
                let res = external_client.append_entries(follower_id.clone(), msg);
                if let Ok(res) = res {
                    internal_client.send(RequestMessage::AppendEntriesResult(
                        leader_term,
                        follower_id,
                        last_log_index,
                        res,
                    ));
                }
            });
        }
    }

    fn finish_joint_consensus(&mut self) {
        self.state.finish_joint_consensus();
        if !self.state.in_cluster() {
            if self.state.in_non_voting_members() {
                self.change_role(Role::NONVOTER);
            } else {
                self.change_role(Role::SHUTDOWN);
            }
        }
    }
}
