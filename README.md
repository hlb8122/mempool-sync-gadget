# Mempool Synchronization Gadget

Using a combination of ZeroMQ and RPC the gadget communicates with another instance of itself and allows two nodes mempools to be synced with near minimal bandwidth in 2 round trips.

## Prerequisites

```bash
sudo apt install libzmq3-dev
sudo apt install libssl-dev
```

The `minisketch-rs` library seems to try build using a Linux only header file. As a result Linux is required to build.

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
mempool-sync-gadget -peerip X.X.X.X -port Y -rpcusername xxxxxx -rpcpassword yyyyyy
```

where `X.X.X.X:Y` is the address of machine A and `z` is the length of the heartbeat period in milliseconds.

For more options `mempool-sync-gadget --help`.
