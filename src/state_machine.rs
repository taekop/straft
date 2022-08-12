use crate::Command;

pub trait StateMachineClient: 'static + Clone + Send {
    fn execute(&mut self, command: Command);
}
