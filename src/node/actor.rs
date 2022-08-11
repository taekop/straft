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
    Command, NodeConfig, NodeId,
};

#[derive(Debug)]
pub struct Request<C: Command> {
    pub msg: RequestMessage<C>,
    pub sender: Option<mpsc::Sender<ResponseMessage>>,
}

#[derive(Debug)]
pub enum RequestMessage<C: Command> {
    // rpc, called by external
    AppendEntries(AppendEntriesRequest<C>),
    RequestVote(RequestVoteRequest),
    AppendLog(AppendLogRequest<C>),
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
            sender.send(res).expect("Failed to send response");
        }
    }
}
