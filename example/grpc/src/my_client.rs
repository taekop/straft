// RPC(gRPC) client provider for Raft Node
// Send request to other nodes

use anyhow::{bail, Result};
use tokio::runtime::Builder;

use crate::grpc::{raft_client::RaftClient, AppendEntriesRequest, RequestVoteRequest};

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
    fn write(&mut self, _request: straft::rpc::WriteRequest) -> Result<straft::rpc::WriteResponse> {
        bail!("should not be used");
    }
    fn read(&mut self, _request: straft::rpc::ReadRequest) -> Result<straft::rpc::ReadResponse> {
        bail!("should not be used");
    }
}

impl straft::RPCClient for MyClient {}
