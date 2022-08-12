// RPC(gRPC) server provider for Raft Node
// Get request from either external client or other nodes

use tonic::{transport::Server, Request, Response, Status};

use crate::grpc::{
    raft_server::{Raft, RaftServer},
    AppendEntriesRequest, AppendEntriesResponse, ReadRequest, ReadResponse, RequestVoteRequest,
    RequestVoteResponse, WriteRequest, WriteResponse,
};
use straft::NodeClient;

pub struct App {
    pub client: NodeClient,
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

    async fn write(
        &self,
        request: Request<WriteRequest>,
    ) -> Result<Response<WriteResponse>, Status> {
        let client = self.client.clone();
        let request = straft::RequestMessage::Write(request.into_inner().into());
        let response = client.send(request);
        match response {
            straft::ResponseMessage::Write(response) => Ok(Response::new(response.into())),
            _ => Err(Status::internal("Invalid response type")),
        }
    }

    async fn read(&self, request: Request<ReadRequest>) -> Result<Response<ReadResponse>, Status> {
        let client = self.client.clone();
        let request = straft::RequestMessage::Read(request.into_inner().into());
        let response = client.send(request);
        match response {
            straft::ResponseMessage::Read(response) => Ok(Response::new(response.into())),
            _ => Err(Status::internal("Invalid response type")),
        }
    }
}
