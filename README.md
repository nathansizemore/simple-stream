# simple-stream [<img src="https://travis-ci.org/nathansizemore/simple-stream.png?branch=master">][travis-badge]

[Documentation][docs]

---
The simple-stream crate provides simple abstraction building blocks over any type that implements [`std::io::Read`][std-io-read] + [`std::io::Write`][std-io-write], coupled with a [`FrameBuilder`][framebuilder]. It provides built-in types for plain text and secured streams with help from [rust-openssl][rust-openssl-repo]. It includes both blocking and non-blocking modes for any type.

It works by handling all of the I/O on a frame based level. It includes a "simple" built-in framing
pattern and WebSocket based framing. It supports custom Frames and FrameBuilders through the frame
module traits.

## Example Usage

~~~rust
extern crate simple_stream as ss;

use ss::frame::Frame;
use ss::frame::simple::{SimpleFrame, SimpleFrameBuilder};
use ss::{Socket, Plain, NonBlocking, SocketOptions};


fn main() {
    // tcp_stream is some connection established std::net::TcpStream
    //
    // Take ownership of the underlying fd to remove TcpStream's Drop being called now
    // that we're switching types.
    let fd = tcp_stream.into_raw_fd();

    // Create a socket and set any POSIX based TCP/SOL_SOCKET options
    let mut socket = Socket::new(fd);
    socket.set_keepalive(true);
    socket.set_nonblocking();

    // Create a plain text based stream that reads messages with SimpleFrame type
    let mut plain_stream = Plain::<Socket, SimpleFrameBuilder>::new(socket);

    // Perform non-blocking read
    match plain_stream.nb_recv() {
        Ok(frames) => {
            // msgs is a Vec<Box<Frame>>
            for frame in frames.iter() {
                // Do stuff with received things
            }
        }
        Err(e) => {
            // Error handling here
        }
    }

    // Perform non-blocking write
    let frame = SimpleFrame::new(&some_buf[..]);
    plain_stream.nb_send(&frame).map_err(|e| {
        // Error handling here
    });
}
~~~

## Author

Nathan Sizemore, nathanrsizemore@gmail.com

## License

simple-stream is available under the MPL-2.0 license. See the LICENSE file for more info.




[travis-badge]: https://travis-ci.org/nathansizemore/simple-stream
[docs]: https://nathansizemore.github.io/simple-stream/simple_stream/index.html
[std-io-read]: https://doc.rust-lang.org/std/io/trait.Read.html
[std-io-write]: https://doc.rust-lang.org/std/io/trait.Write.html
[framebuilder]: https://nathansizemore.github.io/simple-stream/simple_stream/frame/trait.FrameBuilder.html
[rust-openssl-repo]: https://github.com/sfackler/rust-openssl
