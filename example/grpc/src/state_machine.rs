// Simple State Machine
// Actor model based implementation
// Write:   Set value with key
// Read:    Get value by key

use anyhow::{anyhow, Result};
use std::io::prelude::*;
use std::sync::mpsc;
use std::{collections::HashMap, fs::OpenOptions};

#[derive(Debug)]
pub enum MyStateMachineCommand {
    Write(String),
    Read(String),
}

pub struct MyStateMachine {
    path: String,
    state: HashMap<String, String>,
}

impl MyStateMachine {
    pub fn new(path: String) -> MyStateMachine {
        MyStateMachine {
            path,
            state: HashMap::new(),
        }
    }

    pub fn run(
        mut self,
    ) -> mpsc::SyncSender<(MyStateMachineCommand, mpsc::SyncSender<Result<String>>)> {
        let (tx, rx) =
            mpsc::sync_channel::<(MyStateMachineCommand, mpsc::SyncSender<Result<String>>)>(1);
        std::thread::spawn(move || {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(self.path.clone())
                .unwrap();
            loop {
                let req = rx.recv();
                match req {
                    Ok((cmd, tx)) => {
                        // for debugging state machine
                        writeln!(file, "{:?}", cmd).ok();
                        let res: Result<String> = match cmd {
                            MyStateMachineCommand::Read(cmd) => match self.state.get(&cmd) {
                                Some(value) => Ok(value.clone()),
                                None => Err(anyhow!("Key not found")),
                            },
                            MyStateMachineCommand::Write(cmd) => {
                                let array = cmd.split(' ').collect::<Vec<&str>>();
                                if array.len() != 2 {
                                    Err(anyhow!("Usage: <key> <value>"))
                                } else {
                                    let key = array[0].to_string();
                                    let value = array[1].to_string();
                                    self.state.insert(key, value);
                                    Ok(format!(""))
                                }
                            }
                        };
                        tx.send(res).ok();
                    }
                    Err(_) => break,
                }
            }
            std::process::exit(1);
        });
        tx
    }
}

#[derive(Clone)]
pub struct MyStateMachineClient {
    pub tx: mpsc::SyncSender<(MyStateMachineCommand, mpsc::SyncSender<Result<String>>)>,
}

impl straft::StateMachineClient for MyStateMachineClient {
    fn write(&self, command: String) -> Result<String> {
        let (tx, rx) = mpsc::sync_channel(1);
        self.tx.send((MyStateMachineCommand::Write(command), tx))?;
        let result = rx.recv()?;
        result
    }

    fn read(&self, command: String) -> Result<String> {
        let (tx, rx) = mpsc::sync_channel(1);
        self.tx.send((MyStateMachineCommand::Read(command), tx))?;
        let result = rx.recv()?;
        result
    }
}
