use straft::node::Node;
use straft::rpc::RPC;
use tonic::{transport::Server, Request, Response, Status};
use crate::grpc::{raft_server::{Raft, RaftServer}, AppendEntriesRequest, AppendEntriesResponse, RequestVoteRequest, RequestVoteResponse, AppendLogRequest, AppendLogResponse};
use crate::types::{MyCommand, MyExecutor};

pub struct App {
    pub node: Node<MyCommand, MyExecutor>,
    pub addr: std::net::SocketAddr,
}

impl App {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = self.addr.clone();
        self.node.log_info("Running...");

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
