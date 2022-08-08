use raft_client::RaftClient;
use tonic::transport::Channel;

tonic::include_proto!("raft");

async fn append_entries(
    client: &mut RaftClient<Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(AppendEntriesRequest {
        term: Some(0),
        leader_id: Some(String::from("")),
        prev_log_index: Some(0),
        prev_log_term: Some(0),
        entries: Vec::new(),
        leader_commit: Some(0),
    });

    let response = client.append_entries(request).await?;

    println!("AppendEntries RESPONSE={:?}", response);

    Ok(())
}

async fn request_vote(client: &mut RaftClient<Channel>) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(RequestVoteRequest {
        term: Some(0),
        candidate_id: Some(String::from("")),
        last_log_index: Some(0),
        last_log_term: Some(0),
    });

    let response = client.request_vote(request).await?;

    println!("RequestVote RESPONSE={:?}", response);

    Ok(())
}

async fn append_log(client: &mut RaftClient<Channel>) -> Result<(), Box<dyn std::error::Error>> {
    let request = tonic::Request::new(AppendLogRequest {
        command: Some(String::from("")),
    });

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
