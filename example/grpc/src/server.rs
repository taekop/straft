#[macro_use]
extern crate slog;
extern crate slog_term;

use slog::Drain;
use std::collections::{HashMap, HashSet};
use std::io::{self, Write};
use straft::{ClusterConfig, Logger, Node};

mod app;
mod grpc;
mod my_client;
mod state_machine;

use app::App;
use my_client::MyClient;
use state_machine::{MyStateMachine, MyStateMachineClient};

fn get_number() -> usize {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let n: usize = input.trim().parse().expect("Failed to parse number");
    n
}

fn get_config(num: usize) -> (String, String, ClusterConfig, MyClient) {
    let ids = vec![
        String::from("alpha"),
        String::from("beta"),
        String::from("gamma"),
    ];
    let addrs = vec![
        String::from("http://[::1]:50050"),
        String::from("http://[::1]:50052"),
        String::from("http://[::1]:50051"),
    ];
    let id_addr: HashMap<String, String> = ids.iter().cloned().zip(addrs.iter().cloned()).collect();
    let external_client = MyClient::new(id_addr);

    let id = ids[num].clone();
    let addr = addrs[num].clone();
    let config = ClusterConfig {
        members: HashSet::from_iter(ids.into_iter()),
        election_timeout: 1000..2000,
        heartbeat_period: 200,
        majority: 2,
    };
    (id, addr, config, external_client)
}

fn get_state_machine(id: &str) -> MyStateMachineClient {
    let state_machine = MyStateMachine::new(format!("log/log-{}.txt", id));
    MyStateMachineClient {
        tx: state_machine.run(),
    }
}

fn get_logger(id: &str, level: slog::Level) -> Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let async_drain = slog_async::Async::new(slog::LevelFilter::new(drain, level).fuse())
        .build()
        .fuse();
    let root_logger = slog::Logger::root(
        async_drain,
        o!("desc" => "Straft gRPC Example", "version" => "0.1.0"),
    );
    let server_logger = root_logger.new(o!("node" => format!("{id}")));
    server_logger
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    print!("Node Number (0~2): ");
    io::stdout().flush().expect("Failed to flush");
    let node_number = get_number();
    let (id, addr, config, external_client) = get_config(node_number);

    let state_machine_client = get_state_machine(&id);

    let logger = get_logger(&id, slog::Level::Debug);

    let client = Node::<MyStateMachineClient, MyClient>::run(
        id,
        config,
        state_machine_client,
        logger,
        external_client,
    );

    let app = App {
        client,
        addr: addr[7..].parse().expect("Failed to parse addr"),
    };
    app.run().await?;
    Ok(())
}
