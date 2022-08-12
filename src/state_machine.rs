use anyhow::Result;

pub trait StateMachineClient: 'static + Clone + Send {
    fn write(&self, command: String) -> Result<String>;
    fn read(&self, command: String) -> Result<String>;
}
