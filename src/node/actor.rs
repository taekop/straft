use std::thread::sleep;
use std::time::Duration;
use std::{collections::HashSet, sync::mpsc};

use crate::{
    node::{
        client::{ExternalNodeClient, InternalNodeClient},
        election_timer::ElectionTimer,
        logger::Logger,
        state::NodeState,
        Node,
    },
    rpc::{
        AppendEntriesRequest, AppendEntriesResponse, ChangeMembershipRequest,
        ChangeMembershipResponse, InstallSnapshotRequest, InstallSnapshotResponse, ReadRequest,
        ReadResponse, RequestVoteRequest, RequestVoteResponse, WriteRequest, WriteResponse,
    },
    state_machine::StateMachineClient,
    ClusterConfig, NodeId,
};

#[derive(Debug)]
pub struct Request {
    pub msg: RequestMessage,
    pub sender: Option<mpsc::Sender<ResponseMessage>>,
}

#[derive(Debug)]
pub enum RequestMessage {
    // called by other node
    AppendEntries(AppendEntriesRequest),
    RequestVote(RequestVoteRequest),
    // called by client
    ChangeMembership(ChangeMembershipRequest),
    // called by other node
    InstallSnapshot(InstallSnapshotRequest),
    // called by client
    Write(WriteRequest),
    Read(ReadRequest),
    // called by self
    Heartbeat,
    AppendEntriesResult(u64, NodeId, usize, AppendEntriesResponse), // leader term, follower id, last log index
    RequestVoteResult(u64, NodeId, RequestVoteResponse),            // candidate term, follower id
}

#[derive(Debug)]
pub enum ResponseMessage {
    AppendEntries(AppendEntriesResponse),
    RequestVote(RequestVoteResponse),
    ChangeMembership(ChangeMembershipResponse),
    InstallSnapshot(InstallSnapshotResponse),
    Write(WriteResponse),
    Read(ReadResponse),
    Heartbeat,
    AppendEntriesResult,
    RequestVoteResult,
    // async request, wait for state machine
    WriteResult(mpsc::Receiver<ResponseMessage>),
    ReadResult(mpsc::Receiver<ResponseMessage>),
}

impl<SM: StateMachineClient, Client: ExternalNodeClient> Node<SM, Client> {
    pub fn run(
        id: NodeId,
        members: HashSet<NodeId>,
        config: ClusterConfig,
        state_machine: SM,
        logger: Logger,
        external_client: Client,
    ) -> InternalNodeClient {
        let (tx, rx) = mpsc::sync_channel(32);

        let internal_client = InternalNodeClient::new(tx);
        let _internal_client = internal_client.clone();
        let state_machine = state_machine.clone();

        std::thread::spawn(move || {
            let node = Node::new(
                id,
                members,
                config,
                state_machine,
                logger,
                external_client,
                internal_client,
                rx,
            );
            node._run();
        });
        _internal_client
    }

    fn new(
        id: NodeId,
        members: HashSet<NodeId>,
        config: ClusterConfig,
        state_machine: SM,
        logger: Logger,
        external_client: Client,
        internal_client: InternalNodeClient,
        receiver: mpsc::Receiver<Request>,
    ) -> Node<SM, Client> {
        let minimum_election_timeout = config.minimum_election_timeout;
        let maximum_election_timeout = config.maximum_election_timeout;
        Node {
            id: id.clone(),
            config,
            state_machine,
            logger,
            external_client,
            internal_client,
            receiver,
            state: NodeState::new(id, members),
            election_timer: ElectionTimer::new(minimum_election_timeout, maximum_election_timeout),
            running: true,
        }
    }

    fn _run(mut self) {
        self.log_info(format!("Running..."));
        self.election_timer.reset(false);

        // heartbeat
        let heartbeat_sender = self.internal_client.clone();
        std::thread::spawn(move || loop {
            heartbeat_sender.clone().send(RequestMessage::Heartbeat);
            sleep(Duration::from_millis(self.config.heartbeat_period));
        });

        // actor
        while self.running {
            let req = self.receiver.recv();
            match req {
                Ok(req) => {
                    self.handle(req);
                }
                Err(_) => break,
            }
        }
    }

    pub fn handle(&mut self, req: Request) {
        self.log_debug(format!("Got Request: {:?}", req.msg));
        let res = match req.msg {
            RequestMessage::AppendEntries(msg) => {
                ResponseMessage::AppendEntries(self.handle_append_entries(msg))
            }
            RequestMessage::RequestVote(msg) => {
                ResponseMessage::RequestVote(self.handle_request_vote(msg))
            }
            RequestMessage::ChangeMembership(msg) => {
                ResponseMessage::ChangeMembership(self.handle_change_membership(msg))
            }
            RequestMessage::InstallSnapshot(msg) => {
                ResponseMessage::InstallSnapshot(self.handle_install_snapshot(msg))
            }
            RequestMessage::Write(msg) => ResponseMessage::WriteResult(self.handle_write(msg)),
            RequestMessage::Read(msg) => ResponseMessage::ReadResult(self.handle_read(msg)),
            RequestMessage::Heartbeat => self.handle_heartbeat(),
            RequestMessage::AppendEntriesResult(leader_term, follower_id, last_log_index, msg) => {
                self.handle_append_entries_result(leader_term, follower_id, last_log_index, msg)
            }
            RequestMessage::RequestVoteResult(follower_id, candidate_term, msg) => {
                self.handle_request_vote_result(candidate_term, follower_id, msg)
            }
        };
        if let Some(sender) = req.sender {
            match res {
                // R/W requests need to wait for state machine
                ResponseMessage::WriteResult(rx) | ResponseMessage::ReadResult(rx) => {
                    std::thread::spawn(move || {
                        if let Ok(msg) = rx.recv() {
                            sender.send(msg).ok();
                        }
                    });
                }
                res => {
                    sender.send(res).ok();
                }
            };
        }
    }

    pub fn shutdown(&mut self) {
        self.log_info(format!("Shutdown."));
        self.running = false;
    }
}
