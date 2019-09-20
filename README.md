# ipfs_log_rs

> A Rust implementation of [ipfs-log](https://github.com/orbitdb/ipfs-log) by Haja Networks: an append-only log on IPFS.

`ipfs_log_rs` is an immutable conflict-free replicated data structure ([CRDT](https://en.wikipedia.org/wiki/CRDT)) for distributed systems. It is an append-only log that can be used to model a mutable, shared state between peers in peer-to-peer applications.

Every entry in the log is stored in [IPFS](https://ipfs.io), and each points to a hash/hashes of the previous entry/entries, forming a graph. Logs can be forked and joined back together.

*An example graph from the original implementation readme:*
```
           Log A                Log B
             |                    |
     logA.append("one")   logB.append("hello")
             |                    |
             v                    v
          +-----+             +-------+
          |"one"|             |"hello"|
          +-----+             +-------+
             |                    |
     logA.append("two")   logB.append("world")
             |                    |
             v                    v
       +-----------+       +---------------+
       |"one","two"|       |"hello","world"|
       +-----------+       +---------------+
             |                    |
             |                    |
       logA.join(logB) <----------+
             |
             v
+---------------------------+
|"one","hello","two","world"|
+---------------------------+
```

## Status

*Pre-alpha*: only local functionality, IPFS storing unimplemented as of yet.

**Not valid for actual use at the moment.**

## Requirements

* Rust 2018 edition

## Tests

```
cargo test
```

## Copyright

&copy; 2016&ndash;2018 Protocol Labs Inc.,  
2016&ndash;2019 Haja Networks Oy,  
2019 Equilibrium Labs.
