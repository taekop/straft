use crate::Command;

pub trait Executor<C: Command>: Send + Sync{
    fn execute(&self, command: C) {
        println!("Execute {:?}", command);
    }
}
