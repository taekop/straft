use std::sync::mpsc;

use crate::node::actor::{Request, RequestMessage, ResponseMessage};

#[derive(Clone)]
pub struct NodeClient {
    sender: mpsc::SyncSender<Request>,
}

impl NodeClient {
    pub fn new(sender: mpsc::SyncSender<Request>) -> NodeClient {
        NodeClient { sender }
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
