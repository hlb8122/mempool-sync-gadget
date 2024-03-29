# Mempool Synchronization Gadget

A gadget communicates with its accompanying Bitcoin node via ZeroMQ and RPC, and with other gadgets via raw TCP in order to syncronize mempools with near minimal bandwidth in 2 round trips. If the the mempools become sync'd part way through the protocol it terminates, giving a lower than 2 round trip average.

[![Build Status](https://travis-ci.com/hlb8122/mempool-sync-gadget.svg?branch=master)](https://travis-ci.com/hlb8122/mempool-sync-gadget)

## Prerequisites

```bash
sudo apt install clang libzmq3-dev libssl-dev
```

### ZeroMQ

Start Bitcoin's ZeroMQ publishers using 

```bitcoind -zmqpubhashblock=tcp://127.0.0.1:28332 -zmqpubrawtx=tcp://127.0.0.1:28332```
  
or by adding

```properties
zmqpubhashblock=tcp://127.0.0.1:28332
zmqpubrawtx=tcp://127.0.0.1:28332
```

to your `bitcoin.conf`.

### RPC

Allow Bitcoin to accept RPC connections. [See here](https://bitcoin.org/en/developer-reference#remote-procedure-calls-rpcs)

## Running

Once two nodes are running, both with RPC and ZMQ enabled:

### Machine A

```bash
cargo build --release
cd target/release
mempool-sync-gadget --rpcusername xxxxxx --rpcpassword yyyyyy
```

### Machine B

```bash
cargo build --release
cd target/release
mempool-sync-gadget --peerip X.X.X.X --peerport Y --rpcusername xxxxxx --rpcpassword yyyyyy
```

where `X.X.X.X:Y` is the address of machine A.

For more options `mempool-sync-gadget --help`.

## Notes

* All communication is asynchronous, however blocking does occur due to the mutex locking around the gadgets mempool.
* The `minisketch-rs` library seems to try build using a Linux only header file.
* If a gadget receives a transaction from another gadget, sends it to the mempool just after the same transaction arrives at the node `Rpc(Object({"code": Number(-25), "message": String("Missing inputs")}))` error will be thrown and handled gracefully.
