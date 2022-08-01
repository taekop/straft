use tonic::{transport::Server, Request, Response, Status};

use straft::raft_server::{Raft, RaftServer};
use straft::{
    AppendEntriesRequest, AppendEntriesResponse, AppendLogRequest, AppendLogResponse,
    RequestVoteRequest, RequestVoteResponse,
};

pub mod straft {
    tonic::include_proto!("raft");
}

#[derive(Debug, Default)]
pub struct Node {}

#[tonic::async_trait]
impl Raft for Node {
    async fn append_entries(
        &self,
        request: Request<AppendEntriesRequest>,
    ) -> Result<Response<AppendEntriesResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = straft::AppendEntriesResponse {
            term: Some(1),
            success: Some(true),
        };

        Ok(Response::new(reply))
    }

    async fn request_vote(
        &self,
        request: Request<RequestVoteRequest>,
    ) -> Result<Response<RequestVoteResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = straft::RequestVoteResponse {
            term: Some(1),
            vote_granted: Some(true),
        };

        Ok(Response::new(reply))
    }

    async fn append_log(
        &self,
        request: Request<AppendLogRequest>,
    ) -> Result<Response<AppendLogResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = straft::AppendLogResponse {
            success: Some(true),
            leader_id: None,
            leader_address: None,
        };

        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let node = Node::default();

    Server::builder()
        .add_service(RaftServer::new(node))
        .serve(addr)
        .await?;

    Ok(())
}
