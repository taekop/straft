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
    SaveSnapshot(usize, u64),
    InstallSnapshot(Vec<u8>, usize, bool, usize, u64),
}

pub struct MyStateMachine {
    id: String,
    state: HashMap<String, String>,
    snapshot_dir: String,
    log_path: String,
}

impl MyStateMachine {
    pub fn new(id: String, snapshot_dir: String, log_path: String) -> MyStateMachine {
        MyStateMachine {
            id,
            state: HashMap::new(),
            snapshot_dir,
            log_path,
        }
    }

    pub fn snapshot_path(&self, last_included_index: usize, last_included_term: u64) -> String {
        format!(
            "{}/snapshot_{}_{}_{}.json",
            self.snapshot_dir, self.id, last_included_index, last_included_term
        )
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
                .open(self.log_path.clone())
                .unwrap();
            loop {
                let req = rx.recv();
                match req {
                    Ok((cmd, tx)) => {
                        // for debugging state machine
                        writeln!(file, "Got command: {:?}", cmd).ok();
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
                            MyStateMachineCommand::SaveSnapshot(
                                last_included_index,
                                last_included_term,
                            ) => {
                                let snapshot_path =
                                    self.snapshot_path(last_included_index, last_included_term);
                                let snapshot_file = OpenOptions::new()
                                    .create(true)
                                    .write(true)
                                    .open(snapshot_path.clone())
                                    .unwrap();
                                serde_json::to_writer_pretty(snapshot_file, &self.state).unwrap();
                                Ok(snapshot_path)
                            }
                            MyStateMachineCommand::InstallSnapshot(
                                data,
                                _offset,
                                done,
                                last_included_index,
                                last_included_term,
                            ) => {
                                let snapshot_path =
                                    self.snapshot_path(last_included_index, last_included_term);
                                let mut snapshot_file = OpenOptions::new()
                                    .create(true)
                                    .read(true)
                                    .write(true)
                                    .append(true)
                                    .open(snapshot_path)
                                    .unwrap();
                                snapshot_file.write(&data).unwrap();
                                if done {
                                    let state = serde_json::from_reader(snapshot_file).unwrap();
                                    writeln!(file, "Install snapshot {:?}", state).ok();
                                    self.state = state;
                                }
                                Ok(format!(""))
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

    fn save_snapshot(&self, last_included_index: usize, last_included_term: u64) -> Result<String> {
        let (tx, rx) = mpsc::sync_channel(1);
        self.tx.send((MyStateMachineCommand::SaveSnapshot(last_included_index, last_included_term), tx))?;
        let result = rx.recv()?;
        result
    }

    fn install_snapshot(
        &self,
        data: Vec<u8>,
        offset: usize,
        done: bool,
        last_included_index: usize,
        last_included_term: u64,
    ) -> Result<String> {
        let (tx, rx) = mpsc::sync_channel(1);
        self.tx
            .send((
                MyStateMachineCommand::InstallSnapshot(
                    data,
                    offset,
                    done,
                    last_included_index,
                    last_included_term,
                ),
                tx,
            ))
            .unwrap();
        let result = rx.recv()?;
        result
    }
}
