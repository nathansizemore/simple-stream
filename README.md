# simple-stream

Async I/O Wrapper for [TcpStream](https://doc.rust-lang.org/stable/std/net/struct.TcpStream.html) designed to be used with [epoll](https://github.com/nathansizemore/epoll) with a simple data framing schema.

### Data Framing
```
0                   1                   2                   3
0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|          Payload Len        |           Payload           |
+-----------------------------------------------------------+
|                   Payload Data Continued                  |
+-----------------------------------------------------------+

Payload Len:    16 bits
Payload Data:   (Payload Len) bytes
```

### Usage

Add the following to your `Cargo.toml`

```toml
[dependencies.simple-stream]
git = "https://github.com/nathansizemore/simple-stream"
```

Add the following to your crate root

```rust
extern crate simple_stream;
```
