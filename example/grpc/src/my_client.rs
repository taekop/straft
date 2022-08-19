// RPC(gRPC) client provider for Raft Node
// Send request to other nodes

use std::collections::HashMap;

use anyhow::Result;
use tokio::runtime::Builder;

use crate::grpc::{raft_client::RaftClient, AppendEntriesRequest, RequestVoteRequest, InstallSnapshotRequest};

#[derive(Clone)]
pub struct MyClient {
    pub addr: HashMap<String, String>,
}

impl MyClient {
    pub fn new(addr: HashMap<String, String>) -> MyClient {
        MyClient { addr: addr }
    }
}

impl straft::ExternalNodeClient for MyClient {
    fn append_entries(
        &mut self,
        id: String,
        request: straft::rpc::AppendEntriesRequest,
    ) -> Result<straft::rpc::AppendEntriesResponse> {
        let addr = self.addr[&id].clone();
        let rt = Builder::new_multi_thread().enable_all().build()?;
        let mut client = rt.block_on(RaftClient::connect(addr))?;
        let response = rt.block_on(client.append_entries(AppendEntriesRequest::from(request)))?;
        Ok(response.into_inner().into())
    }

    fn request_vote(
        &mut self,
        id: String,
        request: straft::rpc::RequestVoteRequest,
    ) -> Result<straft::rpc::RequestVoteResponse> {
        let addr = self.addr[&id].clone();
        let rt = Builder::new_multi_thread().enable_all().build()?;
        let mut client = rt.block_on(RaftClient::connect(addr))?;
        let response = rt.block_on(client.request_vote(RequestVoteRequest::from(request)))?;
        Ok(response.into_inner().into())
    }

    fn install_snapshot(
        &mut self,
        id: straft::NodeId,
        request: straft::rpc::InstallSnapshotRequest,
    ) -> Result<straft::rpc::InstallSnapshotResponse> {
        let addr = self.addr[&id].clone();
        let rt = Builder::new_multi_thread().enable_all().build()?;
        let mut client = rt.block_on(RaftClient::connect(addr))?;
        let response = rt.block_on(client.install_snapshot(InstallSnapshotRequest::from(request)))?;
        Ok(response.into_inner().into())
    }
}
