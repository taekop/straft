use crate::node::Node;

use tonic::{transport::Server, Request, Response, Status};

tonic::include_proto!("raft");

use raft_server::{Raft, RaftServer};

impl Node {
    pub async fn start(self, addr: std::net::SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        self.log_info("Running...");

        Server::builder()
            .add_service(RaftServer::new(self))
            .serve(addr)
            .await?;
        Ok(())
    }
}

#[tonic::async_trait]
impl Raft for Node {
    async fn append_entries(
        &self,
        request: Request<AppendEntriesRequest>,
    ) -> Result<Response<AppendEntriesResponse>, Status> {
        self.log_debug(&format!("gRPC Request: {:?}", request));

        let reply = AppendEntriesResponse {
            term: Some(1),
            success: Some(true),
        };

        Ok(Response::new(reply))
    }

    async fn request_vote(
        &self,
        request: Request<RequestVoteRequest>,
    ) -> Result<Response<RequestVoteResponse>, Status> {
        self.log_debug(&format!("gRPC Request: {:?}", request));

        let reply = RequestVoteResponse {
            term: Some(1),
            vote_granted: Some(true),
        };

        Ok(Response::new(reply))
    }

    async fn append_log(
        &self,
        request: Request<AppendLogRequest>,
    ) -> Result<Response<AppendLogResponse>, Status> {
        self.log_debug(&format!("gRPC Request: {:?}", request));

        let reply = AppendLogResponse {
            success: Some(true),
            leader_id: None,
            leader_address: None,
        };

        Ok(Response::new(reply))
    }
}
