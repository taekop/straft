#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate slog;
extern crate slog_term;

use slog::Drain;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;
use straft::{Logger, Node, NodeConfig};
use tokio::sync::Mutex;

mod app;
mod grpc;
mod types;

use app::App;
use grpc::raft_client::RaftClient;
use types::{MyClient, MyCommand, MyExecutor};

fn logger(id: &str, level: slog::Level) -> Logger {
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
    let id = String::from("alpha");
    let config = NodeConfig::<MyCommand, MyClient> {
        id: id.clone(),
        client: HashMap::from([(
            id.clone(),
            Arc::new(Mutex::new(MyClient::new(String::from("http://[::1]:50051")))),
        )]),
        election_timeout: 1000..2000,
        heartbeat_period: 1000,
        majority: 1,
        _phantom_c: PhantomData,
    };
    let addr = "[::1]:50051".parse()?;
    let executor = MyExecutor {};
    let logger = logger(&id, slog::Level::Debug);

    let node = Node::new(config, executor, logger);
    let app = App {
        node: Arc::new(node),
        addr: addr,
    };
    app.run().await?;

    Ok(())
}
