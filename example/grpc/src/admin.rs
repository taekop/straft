use raft_client::RaftClient;
use std::{
    collections::HashSet,
    io::{self, Write},
};
use tonic::transport::Channel;

tonic::include_proto!("raft");

async fn change_membership(
    client: &mut RaftClient<Channel>,
    members: HashSet<String>,
) -> Result<tonic::Response<ChangeMembershipResponse>, Box<dyn std::error::Error>> {
    let request = tonic::Request::new(ChangeMembershipRequest {
        members: members.into_iter().collect(),
    });
    let response = client.change_membership(request).await?;
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

fn get_array() -> Vec<usize> {
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .expect("Failed to read line");
    line.trim()
        .to_string()
        .split(" ")
        .map(|s| s.parse().unwrap())
        .collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print!("Node number to send request: ");
    io::stdout().flush().expect("Failed to flush");
    let n = get_number();
    let port = n + 50050;
    let addr = format!("http://[::1]:{}", port);
    let mut client = RaftClient::connect(addr).await.expect("Failed to connect");

    print!("Enter new membership (ex) 0 1 4: ");
    io::stdout().flush().expect("Failed to flush");
    let numbers = get_array();
    let members = numbers
        .into_iter()
        .map(|i| match i {
            0 => String::from("alpha"),
            1 => String::from("beta"),
            2 => String::from("gamma"),
            3 => String::from("delta"),
            4 => String::from("epsilon"),
            _ => panic!("Invalid node number (0~4)"),
        })
        .collect();
    
    let response = change_membership(&mut client, members).await?.into_inner();
    println!("{response:?}");
    Ok(())
}
