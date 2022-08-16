use std::sync::mpsc;
use std::thread::sleep;
use std::time::Duration;

use crate::{
    node::{
        client::{ExternalNodeClient, InternalNodeClient},
        election_timer::ElectionTimer,
        logger::Logger,
        state::NodeState,
        Node,
    },
    rpc::{
        AppendEntriesRequest, AppendEntriesResponse, ReadRequest, ReadResponse, RequestVoteRequest,
        RequestVoteResponse, WriteRequest, WriteResponse,
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
    // rpc, called by external
    AppendEntries(AppendEntriesRequest),
    RequestVote(RequestVoteRequest),
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
    Write(WriteResponse),
    Read(ReadResponse),
    Heartbeat,
    AppendEntriesResult,
    RequestVoteResult,
    // recursive
    WriteResult(mpsc::Receiver<ResponseMessage>),
    ReadResult(mpsc::Receiver<ResponseMessage>),
}

impl<SM: StateMachineClient, Client: ExternalNodeClient> Node<SM, Client> {
    pub fn run(
        id: NodeId,
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
        config: ClusterConfig,
        state_machine: SM,
        logger: Logger,
        external_client: Client,
        internal_client: InternalNodeClient,
        receiver: mpsc::Receiver<Request>,
    ) -> Node<SM, Client> {
        let election_timeout = config.election_timeout.clone();
        Node {
            id,
            config,
            state_machine,
            logger,
            external_client,
            internal_client,
            receiver,
            state: NodeState::new(),
            election_timer: ElectionTimer::new(election_timeout),
        }
    }

    fn _run(mut self) {
        self.log_info(format!("Running..."));
        self.election_timer.reset();

        // heartbeat
        let heartbeat_sender = self.internal_client.clone();
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

    pub fn handle(&mut self, req: Request) {
        self.log_debug(format!("Got Request: {:?}", req.msg));
        let res = match req.msg {
            RequestMessage::AppendEntries(msg) => {
                ResponseMessage::AppendEntries(self.handle_append_entries(msg))
            }
            RequestMessage::RequestVote(msg) => {
                ResponseMessage::RequestVote(self.handle_request_vote(msg))
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
}
