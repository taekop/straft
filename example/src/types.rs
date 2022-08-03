#[derive(Debug)]
pub struct MyCommand(pub String);

impl straft::Command for MyCommand {}

pub struct MyExecutor {}

impl straft::Executor<MyCommand> for MyExecutor {}
