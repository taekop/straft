use std::sync::mpsc;

use crate::{
    node::actor::{Request, RequestMessage, ResponseMessage},
    Command,
};

#[derive(Clone)]
pub struct NodeClient<C: Command> {
    sender: mpsc::SyncSender<Request<C>>,
}

impl<C: Command> NodeClient<C> {
    pub fn new(sender: mpsc::SyncSender<Request<C>>) -> NodeClient<C> {
        NodeClient { sender }
    }

    pub fn send(&self, msg: RequestMessage<C>) -> ResponseMessage {
        let (tx, rx) = std::sync::mpsc::channel();
        let req = Request {
            msg: msg,
            sender: Some(tx),
        };
        self.sender.send(req).unwrap();
        rx.recv().unwrap()
    }
}
