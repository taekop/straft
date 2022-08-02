#[macro_use]
extern crate slog;
extern crate slog_term;

use slog::Drain;

use straft::node::Node;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let id = "alpha";
    let addr = "[::1]:50051".parse()?;

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::CompactFormat::new(decorator).build().fuse();
    let async_drain = slog_async::Async::new(drain).build().fuse();

    let root_logger = slog::Logger::root(async_drain, o!("desc" => "Straft Example", "version" => "0.1.0"));
    let server_logger = root_logger.new(o!("node" => format!("{id}")));
    
    let node = Node::new(id, server_logger);
    node.start(addr).await?;

    Ok(())
}