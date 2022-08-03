#[macro_use]
extern crate slog;
extern crate slog_term;

use std::sync::Arc;
use slog::Drain;
use straft::node::Node;

mod app;
mod grpc;
mod types;

use app::App;
use types::MyExecutor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let id = String::from("alpha");
    let addr = "[::1]:50051".parse()?;

    let executor = MyExecutor {};

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let async_drain = slog_async::Async::new(drain).build().fuse();
    let root_logger = slog::Logger::root(async_drain, o!("desc" => "Straft gRPC Example", "version" => "0.1.0"));
    let server_logger = root_logger.new(o!("node" => format!("{id}")));

    let node = Node::new(id, executor, server_logger);
    let app = App {node: Arc::new(node), addr: addr};
    app.run().await?;

    Ok(())
}
