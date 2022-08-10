use anyhow::{bail, Result};
use futures::executor::block_on;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::sync::mpsc::{self, SyncSender};
use tokio::runtime::{Builder, Handle, Runtime};
use tonic::transport::Channel;

use crate::grpc::{
    raft_client::RaftClient, AppendEntriesRequest, AppendLogRequest, RequestVoteRequest,
};

#[derive(Debug, Default, Clone)]
pub struct MyCommand(pub String);

impl straft::Command for MyCommand {}

#[derive(Clone)]
pub struct MyClient {
    pub addr: String,
}

impl MyClient {
    pub fn new(mut addr: String) -> MyClient {
        if !addr.starts_with("http://") {
            addr = String::from("http://") + &addr;
        }
        MyClient {
            addr: addr,
        }
    }
}

impl straft::RPC<MyCommand> for MyClient {
    fn append_entries(
        &mut self,
        request: straft::rpc::AppendEntriesRequest<MyCommand>,
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
        request: straft::rpc::AppendLogRequest<MyCommand>,
    ) -> Result<straft::rpc::AppendLogResponse> {
        let addr = self.addr.clone();
        let rt = Builder::new_multi_thread().enable_all().build()?;
        let mut client = rt.block_on(RaftClient::connect(addr))?;
        let response = rt.block_on(client.append_log(AppendLogRequest::from(request)))?;
        Ok(response.into_inner().into())
    }
}

impl straft::RPCClient<MyCommand> for MyClient {}

pub struct MyStateMachine {
    path: String,
}

impl MyStateMachine {
    pub fn new(path: String) -> MyStateMachine {
        MyStateMachine { path }
    }

    pub fn run(self) -> SyncSender<MyCommand> {
        let (tx, rx) = mpsc::sync_channel::<MyCommand>(1);
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
                        writeln!(file, "{}", cmd.0);
                    }
                    Err(_) => break,
                }
            }
        });
        tx
    }
}

#[derive(Clone)]
pub struct MyStateMachineClient {
    pub tx: SyncSender<MyCommand>,
}

impl straft::StateMachineClient<MyCommand> for MyStateMachineClient {
    fn execute(&mut self, command: MyCommand) {
        self.tx.send(command).unwrap();
    }
}
