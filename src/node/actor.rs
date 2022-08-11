use std::sync::mpsc;

use crate::{
    rpc::{
        AppendEntriesRequest, AppendEntriesResponse, AppendLogRequest, AppendLogResponse,
        RequestVoteRequest, RequestVoteResponse,
    },
    Command, NodeId,
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
