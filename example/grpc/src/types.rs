use anyhow::Result;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::sync::mpsc::{self, SyncSender};
use tokio::runtime::Builder;

use crate::grpc::{
    raft_client::RaftClient, AppendEntriesRequest, AppendLogRequest, RequestVoteRequest,
};

#[derive(Clone)]
pub struct MyClient {
    pub addr: String,
}

impl MyClient {
    pub fn new(mut addr: String) -> MyClient {
        if !addr.starts_with("http://") {
            addr = String::from("http://") + &addr;
        }
        MyClient { addr: addr }
    }
}

impl straft::RPC for MyClient {
    fn append_entries(
        &mut self,
        request: straft::rpc::AppendEntriesRequest,
    ) -> Result<straft::rpc::AppendEntriesResponse> {
        let addr = self.addr.clone();
        let rt = Builder::new_multi_thread().enable_all().build()?;
        let mut client = rt.block_on(RaftClient::connect(addr))?;
        let response = rt.block_on(client.append_entries(AppendEntriesRequest::from(request)))?;
        Ok(response.into_inner().into())
    }
    fn request_vote(
        &mut self,
        request: straft::rpc::RequestVoteRequest,
    ) -> Result<straft::rpc::RequestVoteResponse> {
        let addr = self.addr.clone();
        let rt = Builder::new_multi_thread().enable_all().build()?;
        let mut client = rt.block_on(RaftClient::connect(addr))?;
        let response = rt.block_on(client.request_vote(RequestVoteRequest::from(request)))?;
        Ok(response.into_inner().into())
    }
    fn append_log(
        &mut self,
        request: straft::rpc::AppendLogRequest,
    ) -> Result<straft::rpc::AppendLogResponse> {
        let addr = self.addr.clone();
        let rt = Builder::new_multi_thread().enable_all().build()?;
        let mut client = rt.block_on(RaftClient::connect(addr))?;
        let response = rt.block_on(client.append_log(AppendLogRequest::from(request)))?;
        Ok(response.into_inner().into())
    }
}

impl straft::RPCClient for MyClient {}

pub struct MyStateMachine {
    path: String,
}

impl MyStateMachine {
    pub fn new(path: String) -> MyStateMachine {
        MyStateMachine { path }
    }

    pub fn run(self) -> SyncSender<String> {
        let (tx, rx) = mpsc::sync_channel::<String>(1);
        std::thread::spawn(move || {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(self.path.clone())
                .unwrap();
            loop {
                let req = rx.recv();
                match req {
                    Ok(cmd) => {
                        let res = writeln!(file, "{}", cmd);
                        if res.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            std::process::exit(1);
        });
        tx
    }
}

#[derive(Clone)]
pub struct MyStateMachineClient {
    pub tx: SyncSender<String>,
}

impl straft::StateMachineClient for MyStateMachineClient {
    fn execute(&mut self, command: String) {
        self.tx.send(command).unwrap();
    }
}
