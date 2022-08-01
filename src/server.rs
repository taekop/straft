mod node;
use node::Node;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let node = Node::default();
    node.start(addr).await?;

    Ok(())
}
