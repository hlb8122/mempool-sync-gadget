# Mempool Synchronization Toy Model

## Prerequisites

```bash
sudo apt install clang
```

## Example

```bash
cargo run --bin example
```

## Benchmarking

```bash
cargo bench
```

### Results

**Intel(R) Core(TM) i5-8600K CPU @ 3.60GHz | 32 GB RAM**

* [Decode](https://hlb8122.github.io/mempool-sync/oddsketch%20decode/report/index.html)
* [Insert Single](https://hlb8122.github.io/mempool-sync/oddsketch%20insert%20single/report/index.html)
* [Insert Million](https://hlb8122.github.io/mempool-sync/oddsketch%20insert%201%20million/report/index.html)

## Notes

The `minisketch-rs` library seems to try build using a Linux only header file. As a result Linux is required to build.
