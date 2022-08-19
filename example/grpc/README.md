# Guide

Integration: gRPC and HashMap state machine.

[log](./log) : Directory of state machine's log

[snapshot](./snapshot) : Directory of state machine's snapshot


```shell
cargo run --bin server  # Run single raft node
cargo run --bin client  # Send Read/Write request
cargo run --bin admin   # Send ChangeMembership request 
```
