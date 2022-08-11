use tonic::{transport::Server, Request, Response, Status};

use crate::grpc::{
    raft_server::{Raft, RaftServer},
    AppendEntriesRequest, AppendEntriesResponse, AppendLogRequest, AppendLogResponse,
    RequestVoteRequest, RequestVoteResponse,
};
use crate::types::MyCommand;
use straft::NodeClient;

// RPC(gRPC) provider for Raft Node
pub struct App {
    pub client: NodeClient<MyCommand>,
    pub addr: std::net::SocketAddr,
}

impl App {
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = self.addr.clone();

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
        let client = self.client.clone();
        let request = straft::RequestMessage::AppendEntries(request.into_inner().into());
        let response = client.send(request);
        match response {
            straft::ResponseMessage::AppendEntries(response) => Ok(Response::new(response.into())),
            _ => Err(Status::internal("Invalid response type")),
        }
    }

    async fn request_vote(
        &self,
        request: Request<RequestVoteRequest>,
    ) -> Result<Response<RequestVoteResponse>, Status> {
        let client = self.client.clone();
        let request = straft::RequestMessage::RequestVote(request.into_inner().into());
        let response = client.send(request);
        match response {
            straft::ResponseMessage::RequestVote(response) => Ok(Response::new(response.into())),
            _ => Err(Status::internal("Invalid response type")),
        }
    }

    async fn append_log(
        &self,
        request: Request<AppendLogRequest>,
    ) -> Result<Response<AppendLogResponse>, Status> {
        let client = self.client.clone();
        let request = straft::RequestMessage::AppendLog(request.into_inner().into());
        let response = client.send(request);
        match response {
            straft::ResponseMessage::AppendLog(response) => Ok(Response::new(response.into())),
            _ => Err(Status::internal("Invalid response type")),
        }
    }
}
