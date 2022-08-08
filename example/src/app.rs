use std::sync::Arc;
use tonic::{transport::Server, Request, Response, Status};

use crate::grpc::{
    raft_server::{Raft, RaftServer},
    AppendEntriesRequest, AppendEntriesResponse, AppendLogRequest, AppendLogResponse,
    RequestVoteRequest, RequestVoteResponse,
};
use crate::types::{MyClient, MyCommand, MyExecutor};
use straft::node::Node;
use straft::rpc::RPC;

pub struct App {
    pub node: Arc<Node<MyCommand, MyExecutor, MyClient>>,
    pub addr: std::net::SocketAddr,
}

impl App {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = self.addr.clone();
        self.node.log_info(String::from("Running..."));

        let heartbeat = tokio::spawn({
            let node = Arc::clone(&self.node);
            async move {
                node.start_heartbeat();
            }
        });

        Server::builder()
            .add_service(RaftServer::new(self))
            .serve(addr)
            .await?;

        Ok(())
    }
}

#[tonic::async_trait]
impl Raft for App {
    async fn append_entries(
        &self,
        request: Request<AppendEntriesRequest>,
    ) -> Result<Response<AppendEntriesResponse>, Status> {
        let response = self.node.append_entries(request.into_inner().into()).await;
        Ok(Response::new(response.unwrap().into()))
    }

    async fn request_vote(
        &self,
        request: Request<RequestVoteRequest>,
    ) -> Result<Response<RequestVoteResponse>, Status> {
        let response = self.node.request_vote(request.into_inner().into()).await;
        Ok(Response::new(response.unwrap().into()))
    }

    async fn append_log(
        &self,
        request: Request<AppendLogRequest>,
    ) -> Result<Response<AppendLogResponse>, Status> {
        let response = self.node.append_log(request.into_inner().into()).await;
        Ok(Response::new(response.unwrap().into()))
    }
}
