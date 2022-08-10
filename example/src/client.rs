use std::io;
use raft_client::RaftClient;
use tonic::transport::Channel;

tonic::include_proto!("raft");

async fn append_log(client: &mut RaftClient<Channel>, command: String) -> Result<tonic::Response<AppendLogResponse>, Box<dyn std::error::Error>> {
    let request = tonic::Request::new(AppendLogRequest {
        command: Some(command),
    });
    let response = client.append_log(request).await?;
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RaftClient::connect("http://[::1]:50050").await?;
    let mut command = String::new();
    io::stdin().read_line(&mut command)?;
    let response = append_log(&mut client, command).await?.into_inner();
    if response.success.unwrap() {
        println!("Success");
    } else {
        if response.leader_id.is_none() || response.leader_address.is_none() {
            println!("Leader not found");
        } else {
            println!("Retry to leader: {:?} {:?}", response.leader_id.unwrap(), response.leader_address.unwrap());
        }
    }
    Ok(())
}
