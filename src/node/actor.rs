use std::sync::mpsc;
use std::thread::sleep;
use std::time::Duration;

use crate::{
    node::{
        client::NodeClient, election_timer::ElectionTimer, logger::Logger, state::NodeState, Node,
    },
    rpc::{
        AppendEntriesRequest, AppendEntriesResponse, AppendLogRequest, AppendLogResponse,
        RPCClient, RequestVoteRequest, RequestVoteResponse,
    },
    state_machine::StateMachineClient,
    NodeConfig, NodeId,
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
    AppendLog(AppendLogRequest),
    // called by self
    Heartbeat,
    AppendEntriesResult(u64, NodeId, usize, AppendEntriesResponse), // leader term, follower id, last log index
    RequestVoteResult(u64, NodeId, RequestVoteResponse),            // candidate term, follower id
}

#[derive(Debug)]
pub enum ResponseMessage {
    AppendEntries(AppendEntriesResponse),
    RequestVote(RequestVoteResponse),
    AppendLog(AppendLogResponse),
    Heartbeat,
    AppendEntriesResult,
    RequestVoteResult,
}

impl<SM: StateMachineClient, Client: RPCClient> Node<SM, Client> {
    pub fn run(config: NodeConfig<Client>, state_machine: SM, logger: Logger) -> NodeClient {
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
        config: NodeConfig<Client>,
        state_machine: SM,
        logger: Logger,
        receiver: mpsc::Receiver<Request>,
        self_client: NodeClient,
    ) -> Node<SM, Client> {
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

    pub fn handle(&mut self, req: Request) {
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
            sender.send(res).expect("Failed to send response");
        }
    }
}
