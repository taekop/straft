#[macro_use]
extern crate slog;
extern crate slog_term;

use slog::Drain;
use std::collections::HashMap;
use std::io::{self, Write};
use std::marker::PhantomData;
use straft::{Logger, Node, NodeConfig};

mod app;
mod grpc;
mod types;

use app::App;
use types::{MyClient, MyCommand, MyStateMachine, MyStateMachineClient};

fn get_number() -> usize {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let n: usize = input.trim().parse().expect("Failed to parse number");
    n
}

fn get_config(num: usize) -> (String, String, NodeConfig<MyCommand, MyClient>) {
    let ids = vec![
        String::from("alpha"),
        String::from("beta"),
        String::from("gamma"),
    ];
    let addrs = vec![
        String::from("[::1]:50050"),
        String::from("[::1]:50051"),
        String::from("[::1]:50052"),
    ];
    let addresses: HashMap<String, String> =
        ids.iter().cloned().zip(addrs.iter().cloned()).collect();
    let mut client: HashMap<String, MyClient> = ids
        .iter()
        .cloned()
        .zip(addrs.iter().cloned())
        .map(|(id, addr)| (id, MyClient::new(addr)))
        .collect();

    let id = ids[num].clone();
    let addr = addrs[num].clone();
    client.remove(&id.clone());
    let config = NodeConfig {
        id: id.clone(),
        addresses: addresses,
        client: client,
        election_timeout: 1000..2000,
        heartbeat_period: 200,
        majority: 2,
        _phantom_c: PhantomData,
    };
    (id, addr, config)
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
    let (id, addr, config) = get_config(node_number);

    let state_machine_client = get_state_machine(&id);

    let logger = get_logger(&id, slog::Level::Debug);

    let client = Node::<MyCommand, MyStateMachineClient, MyClient>::run(
        config,
        state_machine_client,
        logger,
    );

    let app = App {
        client,
        addr: addr.parse().expect("Failed to parse addr"),
    };
    app.run().await?;
    Ok(())
}
