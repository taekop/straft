use std::cmp::min;
use std::sync::mpsc;
use std::thread::sleep;
use std::time::Duration;

pub mod actor;
pub mod client;
mod election_timer;
pub mod logger;
mod state;

use self::{
    actor::{Request, RequestMessage},
    client::NodeClient,
    election_timer::ElectionTimer,
    logger::Logger,
    state::{NodeState, Role},
};

use crate::{
    rpc::{
        AppendEntriesRequest, AppendEntriesResponse, AppendLogRequest, AppendLogResponse,
        RPCClient, RequestVoteRequest, RequestVoteResponse,
    },
    state_machine::StateMachineClient,
    Command, Entry, NodeConfig, NodeId, ResponseMessage,
};

pub struct Node<C: Command, SM: StateMachineClient<C>, Client: RPCClient<C>> {
    config: NodeConfig<C, Client>,
    election_timer: ElectionTimer,
    logger: Logger,
    receiver: mpsc::Receiver<Request<C>>,
    self_client: NodeClient<C>,
    state: NodeState<C>,
    state_machine: SM,
}

impl<C: Command, SM: StateMachineClient<C>, Client: RPCClient<C>> Node<C, SM, Client> {
    pub fn run(config: NodeConfig<C, Client>, state_machine: SM, logger: Logger) -> NodeClient<C> {
        let (tx, rx) = mpsc::sync_channel(32);

        let client = NodeClient::new(tx);
        let self_client = client.clone();
        let state_machine = state_machine.clone();

        std::thread::spawn(move || {
            let node = Node::new(config, state_machine, logger, rx, self_client);
            node._run();
        });
        client
    }

    fn new(
        config: NodeConfig<C, Client>,
        state_machine: SM,
        logger: Logger,
        receiver: mpsc::Receiver<Request<C>>,
        self_client: NodeClient<C>,
    ) -> Node<C, SM, Client> {
        let election_timeout = config.election_timeout.clone();
        Node {
            config,
            election_timer: ElectionTimer::new(election_timeout),
            logger,
            receiver,
            self_client,
            state: NodeState::new(),
            state_machine,
        }
    }

    fn _run(mut self) {
        self.log_info(format!("Running..."));

        // heartbeat
        let heartbeat_sender = self.self_client.clone();
        std::thread::spawn(move || loop {
            heartbeat_sender.clone().send(RequestMessage::Heartbeat);
            sleep(Duration::from_millis(self.config.heartbeat_period));
        });

        // actor
        loop {
            let req = self.receiver.recv();
            match req {
                Ok(req) => {
                    self.handle(req);
                }
                Err(_) => break,
            }
        }
    }

    pub fn handle(&mut self, req: Request<C>) {
        self.log_debug(format!("Got Request: {:?}", req.msg));
        let res = match req.msg {
            RequestMessage::AppendEntries(msg) => {
                ResponseMessage::AppendEntries(self.handle_append_entries(msg))
            }
            RequestMessage::RequestVote(msg) => {
                ResponseMessage::RequestVote(self.handle_request_vote(msg))
            }
            RequestMessage::AppendLog(msg) => {
                ResponseMessage::AppendLog(self.handle_append_log(msg))
            }
            RequestMessage::Heartbeat => self.handle_heartbeat(),
            RequestMessage::AppendEntriesResult(leader_term, follower_id, last_log_index, msg) => {
                self.handle_append_entries_result(leader_term, follower_id, last_log_index, msg)
            }
            RequestMessage::RequestVoteResult(follower_id, candidate_term, msg) => {
                self.handle_request_vote_result(candidate_term, follower_id, msg)
            }
        };
        if let Some(sender) = req.sender {
            sender.send(res).unwrap();
        }
    }

    fn handle_append_entries(&mut self, req: AppendEntriesRequest<C>) -> AppendEntriesResponse {
        self.election_timer.reset();
        if req.term > self.state.current_term && !self.state.is_role(Role::FOLLOWER) {
            self.change_role(Role::FOLLOWER);
        }

        let valid_term = req.term >= self.state.current_term;
        let valid_log_term = self.state.last_log().term == req.prev_log_term;
        if valid_term && valid_log_term {
            self.state.current_term = req.term;
            self.detect_other_leader(req.leader_id);

            self.state.log.splice(req.prev_log_index + 1.., req.entries);

            if req.leader_commit > self.state.commit_index {
                self.state.commit_index = min(req.leader_commit, self.state.last_log().index);
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
        self.election_timer.reset();
        let current_term = self.state.current_term;
        let voted_for = self.state.voted_for.clone();
        let last_log = self.state.last_log();
        let valid_term = req.term >= current_term;
        let valid_vote = req.term != current_term
            || voted_for.is_none()
            || voted_for.unwrap() == req.candidate_id;
        let valid_log = req.last_log_index >= last_log.index && req.last_log_term >= last_log.term;
        if valid_term && valid_vote && valid_log {
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

    fn handle_append_log(&mut self, req: AppendLogRequest<C>) -> AppendLogResponse {
        if self.state.is_role(Role::LEADER) {
            let last_log = self.state.last_log();
            let new_entry = Entry {
                index: last_log.index + 1,
                term: self.state.current_term,
                command: req.command,
            };
            self.state.log.push(new_entry);
            AppendLogResponse {
                success: true,
                leader_id: self.state.leader_id.clone(),
                leader_address: self.state.leader_address.clone(),
            }
        } else {
            AppendLogResponse {
                success: false,
                leader_id: self.state.leader_id.clone(),
                leader_address: self.state.leader_address.clone(),
            }
        }
    }

    fn handle_heartbeat(&mut self) -> ResponseMessage {
        if self.election_timer.is_timeout() && !self.state.is_role(Role::LEADER) {
            self.change_role(Role::CANDIDATE);
        } else if self.state.is_role(Role::LEADER) {
            self.request_append_entries();
        }

        if self.state.commit_index > self.state.last_applied {
            for entry in
                self.state.log[self.state.last_applied + 1..=self.state.commit_index].iter()
            {
                self.log_debug(format!("Execute: {:?}", entry.command));
                self.state_machine.execute(entry.command.clone());
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

                let majority_match_index: usize = {
                    let mut match_indices = self
                        .state
                        .match_index
                        .values()
                        .cloned()
                        .chain(std::iter::once(last_log_index))
                        .collect::<Vec<usize>>();
                    match_indices.sort();
                    match_indices
                        .get((self.config.majority - 1) as usize)
                        .unwrap_or(&0)
                        .clone()
                };
                if majority_match_index > self.state.commit_index {
                    self.state.commit_index = majority_match_index;
                }
            }
        }
        ResponseMessage::AppendEntriesResult
    }

    fn handle_request_vote_result(
        &mut self,
        _follower_id: NodeId,
        candidate_term: u64,
        req: RequestVoteResponse,
    ) -> ResponseMessage {
        let current_term = self.state.current_term;
        if current_term == candidate_term && self.state.is_role(Role::CANDIDATE) {
            if req.term > current_term {
                self.state.current_term = req.term;
                self.change_role(Role::FOLLOWER);
            } else if req.vote_granted {
                self.state.vote_cnt += 1;
                if self.state.vote_cnt >= self.config.majority {
                    self.change_role(Role::LEADER);
                }
            }
        }
        ResponseMessage::RequestVoteResult
    }

    fn change_role(&mut self, role: Role) {
        self.log_debug(format!("Change role to {:?}", role));
        self.state.change_role(role);
        match role {
            Role::CANDIDATE => {
                self.state.initialize_candidate_state();
                self.detect_no_leader();
                self.run_for_leader();
            }
            Role::LEADER => {
                self.state.initialize_leader_state(
                    self.config.id.clone(),
                    self.config
                        .addresses
                        .get(&self.config.id.clone())
                        .unwrap()
                        .clone(),
                );
                self.request_append_entries();
            }
            _ => {}
        }
    }

    fn run_for_leader(&mut self) {
        let candidate_term = self.state.current_term + 1;
        self.log_debug(format!("Run for leader in term {:?}", candidate_term));
        self.election_timer.reset();
        self.state.current_term = candidate_term;
        self.state.voted_for = Some(self.config.id.clone());

        let last_log = self.state.last_log();
        let msg = RequestVoteRequest {
            term: candidate_term,
            candidate_id: self.config.id.clone(),
            last_log_index: last_log.index,
            last_log_term: last_log.term,
        };
        for (follower_id, client) in self.config.client.iter() {
            let msg = msg.clone();
            let follower_id = follower_id.clone();
            let mut client = client.clone();
            let self_client = self.self_client.clone();
            std::thread::spawn(move || {
                let res = client.request_vote(msg);
                if let Ok(res) = res {
                    self_client.send(RequestMessage::RequestVoteResult(
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
        for (follower_id, client) in self.config.client.iter() {
            let next_index = *self
                .state
                .next_index
                .entry(follower_id.clone())
                .or_insert(last_log_index + 1);
            let prev_log_index = next_index - 1;
            let prev_log_term = self.state.log.get(prev_log_index).unwrap().term;
            let entries = self.state.log[next_index..].to_vec();
            let msg = AppendEntriesRequest {
                term: self.state.current_term,
                leader_id: self.config.id.clone(),
                prev_log_index,
                prev_log_term,
                entries,
                leader_commit: self.state.commit_index,
            };
            let follower_id = follower_id.clone();
            let mut client = client.clone();
            let self_client = self.self_client.clone();
            std::thread::spawn(move || {
                let res = client.append_entries(msg);
                if let Ok(res) = res {
                    self_client.send(RequestMessage::AppendEntriesResult(
                        leader_term,
                        follower_id.clone(),
                        last_log_index,
                        res,
                    ));
                }
            });
        }
    }

    fn detect_no_leader(&mut self) {
        self.state.leader_id = None;
        self.state.leader_address = None;
    }

    fn detect_other_leader(&mut self, leader_id: String) {
        self.state.leader_address = Some(self.config.addresses.get(&leader_id).unwrap().clone());
        self.state.leader_id = Some(leader_id);
    }
}
