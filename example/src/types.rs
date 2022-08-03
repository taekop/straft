#[derive(Debug)]
pub struct Command(pub String);

impl straft::Command for Command {}