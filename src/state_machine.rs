use crate::Command;

pub trait StateMachineClient<C: Command>: 'static + Clone + Send {
    fn execute(&mut self, command: C);
}
