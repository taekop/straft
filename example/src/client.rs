use raft_client::RaftClient;
use tonic::transport::Channel;

tonic::include_proto!("raft");

async fn append_entries(
    client: &mut RaftClient<Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(AppendEntriesRequest {
        term: None,
        leader_id: None,
        prev_log_index: None,
        prev_log_term: None,
        entries: Vec::new(),
        leader_commit: None,
    });

    let response = client.append_entries(request).await?;

    println!("AppendEntries RESPONSE={:?}", response);

    Ok(())
}

async fn request_vote(client: &mut RaftClient<Channel>) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(RequestVoteRequest {
        term: None,
        candidate_id: None,
        last_log_index: None,
        last_log_term: None,
    });

    let response = client.request_vote(request).await?;

    println!("RequestVote RESPONSE={:?}", response);

    Ok(())
}

async fn append_log(client: &mut RaftClient<Channel>) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(AppendLogRequest { command: None });

    let response = client.append_log(request).await?;

    println!("AppendLog RESPONSE={:?}", response);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RaftClient::connect("http://[::1]:50051").await?;

    append_entries(&mut client).await?;
    request_vote(&mut client).await?;
    append_log(&mut client).await?;

    Ok(())
}
