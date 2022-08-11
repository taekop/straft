# Straft

Straft is an implementation of Raft consensus algorithm in Rust.

## Example

### gRPC

```shell
cd example/grpc
cargo run --bin straft-server # enter 0
cargo run --bin straft-server # enter 1
cargo run --bin straft-server # enter 2
cargo run --bin straft-client # enter node number and command
```
