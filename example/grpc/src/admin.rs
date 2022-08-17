use raft_client::RaftClient;
use std::io::{self, Write};
use tonic::transport::Channel;

tonic::include_proto!("raft");

async fn change_members(
    client: &mut RaftClient<Channel>,
    members: Vec<String>,
) -> Result<tonic::Response<ChangeMembershipResponse>, Box<dyn std::error::Error>> {
    let request = tonic::Request::new(ChangeMembershipRequest {
        change_new_members: Some(true),
        new_members: members.into_iter().collect(),
        change_non_voting_members: Some(false),
        non_voting_members: Vec::default(),
    });
    let response = client.change_membership(request).await?;
    Ok(response)
}

async fn change_non_voting_members(
    client: &mut RaftClient<Channel>,
    non_voting_members: Vec<String>,
) -> Result<tonic::Response<ChangeMembershipResponse>, Box<dyn std::error::Error>> {
    let request = tonic::Request::new(ChangeMembershipRequest {
        change_new_members: Some(false),
        new_members: Vec::default(),
        change_non_voting_members: Some(true),
        non_voting_members: non_voting_members.into_iter().collect(),
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

fn get_line() -> String {
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .expect("Failed to read line");
    line.trim().to_string()
}

fn get_array() -> Vec<usize> {
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .expect("Failed to read line");
    let line = line.trim().to_string();
    if line.is_empty() {
        Vec::default()
    } else {
        line.split(" ").map(|s| s.parse().unwrap()).collect()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print!("Node number to send request: ");
    io::stdout().flush().expect("Failed to flush");
    let n = get_number();
    let port = n + 50050;
    let addr = format!("http://[::1]:{}", port);
    let mut client = RaftClient::connect(addr).await.expect("Failed to connect");

    print!("Change members (M) / Change non voting members (N): ");
    io::stdout().flush().expect("Failed to flush");
    let is_change_members = {
        let line = get_line();
        if line.starts_with("M") || line.starts_with("m") {
            true
        } else if line.starts_with("N") || line.starts_with("n") {
            false
        } else {
            panic!("N or M?");
        }
    };

    if is_change_members {
        print!("Enter new members (ex) 0 1 4: ");
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

        let response = change_members(&mut client, members).await?.into_inner();
        println!("{response:?}");
    } else {
        print!("Enter new non voting members (ex) 0 1 4: ");
        io::stdout().flush().expect("Failed to flush");
        let numbers = get_array();
        let non_voting_members = numbers
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

        let response = change_non_voting_members(&mut client, non_voting_members)
            .await?
            .into_inner();
        println!("{response:?}");
    }

    Ok(())
}
