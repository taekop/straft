use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Channel;

use crate::grpc::{
    raft_client::RaftClient, AppendEntriesRequest, AppendLogRequest, RequestVoteRequest,
};

#[derive(Debug, Default)]
pub struct MyCommand(pub String);

impl straft::Command for MyCommand {}

pub struct MyExecutor {}

impl straft::Executor<MyCommand> for MyExecutor {}

pub struct MyClient {
    pub addr: String,
    pub client: Arc<Mutex<Option<RaftClient<Channel>>>>,
}

impl MyClient {
    pub fn new(addr: String) -> MyClient {
        MyClient {
            addr: addr,
            client: Arc::new(Mutex::new(None)),
        }
    }

    // try until connected
    pub async fn get_client(&self) -> Result<RaftClient<Channel>> {
        let mut locked_client = self.client.lock().await;
        let mut client = match *locked_client {
            Some(ref client) => client.clone(),
            None => {
                let mut client = RaftClient::connect(self.addr.clone()).await?;
                *locked_client = Some(client.clone());
                client
            }
        };
        Ok(client)
    }
}

#[async_trait]
impl straft::rpc::RPC<MyCommand> for MyClient {
    async fn append_entries(
        &self,
        request: straft::rpc::AppendEntriesRequest<MyCommand>,
    ) -> Result<straft::rpc::AppendEntriesResponse> {
        let mut client = self.get_client().await?;
        let response = client
            .append_entries(AppendEntriesRequest::from(request))
            .await?;
        Ok(response.into_inner().into())
    }
    async fn request_vote(
        &self,
        request: straft::rpc::RequestVoteRequest,
    ) -> Result<straft::rpc::RequestVoteResponse> {
        let mut client = self.get_client().await?;
        let response = client
            .request_vote(RequestVoteRequest::from(request))
            .await?;
        Ok(response.into_inner().into())
    }
    async fn append_log(
        &self,
        request: straft::rpc::AppendLogRequest<MyCommand>,
    ) -> Result<straft::rpc::AppendLogResponse> {
        let mut client = self.get_client().await?;
        let response = client.append_log(AppendLogRequest::from(request)).await?;
        Ok(response.into_inner().into())
    }
}

impl straft::NodeClient<MyCommand> for MyClient {}
