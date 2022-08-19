use anyhow::Result;
use std::sync::mpsc;

use crate::{
    node::actor::{Request, RequestMessage, ResponseMessage},
    rpc::{
        AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest,
        InstallSnapshotResponse, RequestVoteRequest, RequestVoteResponse,
    },
    NodeId,
};

#[derive(Clone)]
pub struct InternalNodeClient {
    sender: mpsc::SyncSender<Request>,
}

impl InternalNodeClient {
    pub fn new(sender: mpsc::SyncSender<Request>) -> InternalNodeClient {
        InternalNodeClient { sender }
    }

    pub fn send(&self, msg: RequestMessage) -> ResponseMessage {
        let (tx, rx) = std::sync::mpsc::channel();
        let req = Request {
            msg: msg,
            sender: Some(tx),
        };
        self.sender.send(req).unwrap();
        rx.recv().unwrap()
    }
}

pub trait ExternalNodeClient: 'static + Clone + Send + Sync {
    fn append_entries(
        &mut self,
        id: NodeId,
        request: AppendEntriesRequest,
    ) -> Result<AppendEntriesResponse>;
    fn request_vote(
        &mut self,
        id: NodeId,
        request: RequestVoteRequest,
    ) -> Result<RequestVoteResponse>;
    fn install_snapshot(
        &mut self,
        id: NodeId,
        request: InstallSnapshotRequest,
    ) -> Result<InstallSnapshotResponse>;
}
