use raft_client::RaftClient;
use std::io::{self, Write};
use tonic::transport::Channel;

tonic::include_proto!("raft");
async fn write(
    client: &mut RaftClient<Channel>,
    command: String,
) -> Result<tonic::Response<WriteResponse>, Box<dyn std::error::Error>> {
    let request = tonic::Request::new(WriteRequest {
        command: Some(command),
    });
    let response = client.write(request).await?;
    Ok(response)
}

async fn read(
    client: &mut RaftClient<Channel>,
    command: String,
) -> Result<tonic::Response<ReadResponse>, Box<dyn std::error::Error>> {
    let request = tonic::Request::new(ReadRequest {
        command: Some(command),
    });
    let response = client.read(request).await?;
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

fn get_line() -> String {
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .expect("Failed to read line");
    line.trim().to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print!("Node number to send request: ");
    io::stdout().flush().expect("Failed to flush");
    let n = get_number();
    let port = n + 50050;
    let addr = format!("http://[::1]:{}", port);
    let mut client = RaftClient::connect(addr).await?;

    print!("Read or Write(R/W): ");
    io::stdout().flush().expect("Failed to flush");
    let is_read = {
        let line = get_line();
        if line.starts_with("R") || line.starts_with("r") {
            true
        } else if line.starts_with("W") || line.starts_with("w") {
            false
        } else {
            panic!("Read or Write?");
        }
    };

    print!("Enter Command: ");
    io::stdout().flush().expect("Failed to flush");
    let command = get_line();

    if is_read {
        let response = read(&mut client, command).await?.into_inner();
        println!("{response:?}");
    } else {
        let response = write(&mut client, command).await?.into_inner();
        println!("{response:?}");
    }
    Ok(())
}
