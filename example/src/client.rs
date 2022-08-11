use raft_client::RaftClient;
use std::io::{self, Write};
use tonic::transport::Channel;

tonic::include_proto!("raft");

async fn append_log(
    client: &mut RaftClient<Channel>,
    command: String,
) -> Result<tonic::Response<AppendLogResponse>, Box<dyn std::error::Error>> {
    let request = tonic::Request::new(AppendLogRequest {
        command: Some(command),
    });
    let response = client.append_log(request).await?;
    Ok(response)
}

fn get_number() -> usize {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let n: usize = input.trim().parse().expect("Failed to parse number");
    n
}

fn get_command() -> String {
    let mut command = String::new();
    io::stdin()
        .read_line(&mut command)
        .expect("Failed to read line");
    command
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print!("Node Number (0~2): ");
    io::stdout().flush().expect("Failed to flush");
    let n = get_number();
    let port = n + 50050;
    let addr = format!("http://[::1]:{}", port);
    let mut client = RaftClient::connect(addr).await?;

    print!("Enter Command: ");
    io::stdout().flush().expect("Failed to flush");
    let command = get_command();

    let response = append_log(&mut client, command).await?.into_inner();
    if response.success.unwrap() {
        println!("Success");
    } else {
        if response.leader_id.is_none() || response.leader_address.is_none() {
            println!("Leader not found");
        } else {
            println!(
                "Retry to leader: {:?} {:?}",
                response.leader_id.unwrap(),
                response.leader_address.unwrap()
            );
        }
    }
    Ok(())
}
