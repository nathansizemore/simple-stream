# simple-stream

Various wrappers for [TcpStream](https://doc.rust-lang.org/stable/std/net/struct.TcpStream.html). Relies on `read` and `write` syscalls. Very simple data framing pattern.

[NbetStream](https://nathansizemore.github.io/simple-stream/simple_stream/nbetstream/index.html) is an async stream designed to be used with [epoll](https://github.com/nathansizemore/epoll) Linux kernel's epoll in EdgeTriggered mode.

[Documentation](https://nathansizemore.github.io/simple-stream/simple_stream/index.html)

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
