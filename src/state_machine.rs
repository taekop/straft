use anyhow::Result;

pub trait StateMachineClient: 'static + Clone + Send {
    fn write(&self, command: String) -> Result<String>;
    fn read(&self, command: String) -> Result<String>;
    fn install_snapshot(&self, data: Vec<u8>, offset: usize, done: bool, last_included_index: usize, last_included_term: u64) -> Result<String>;
    fn save_snapshot(&self, last_included_index: usize, last_included_term: u64) -> Result<String>;
}
