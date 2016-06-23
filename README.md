# simple-stream [<img src="https://travis-ci.org/nathansizemore/simple-stream.png?branch=master">][travis-badge]

[Documentation][docs]

---

simple-stream is a buffered stream wrapper over anything that implements
`std::io::Read` and `std::io::Write`. It works by buffering all reads and
checking the buffers against a `FrameBuilder`, which will inform the stream
that a complete `Frame` has been received, and removes it out of the buffer.


The crate comes with a few types of Framing options, and provides both a plain
text and encrypted stream via [rust-openssl][rust-openssl-repo].

---

## Example Usage

``` rust
extern crate simple_stream as ss;

use std::net::TcpStream;

use ss::frame::{SimpleFrame, SimpleFrameBuilder};
use ss::{Plain, NonBlocking};


fn main() {
    // Create some non-blocking type that implements Read + Write
    let stream = TcpStream::connect("rust-lang.org:80").unwrap();
    stream.set_nonblocking(true).unwrap();

    // Create a Plain Text stream that sends and receives messages in the
    // `SimpleFrame` format.
    let mut plain_stream = Plain::<TcpStream, SimpleFrameBuilder>::new(stream);

    // Perform a non-blocking write
    let buf = vec!(1, 2, 3, 4);
    let frame = SimpleFrame::new(&buf[..]);
    match plain_stream.nb_send(&frame) {
        Ok(_) => { }
        Err(e) => println!("Error during write: {}", e)
    };

    // Perform a non-blocking read
    match plain_stream.nb_recv() {
        Ok(frames) => {
            for _ in frames {
                // Do stuff with received frames
            }
        }
        Err(e) => println!("Error during read: {}", e)
    };
}
```

---

## Author

Nathan Sizemore, nathanrsizemore@gmail.com


## License

simple-stream is available under the MPL-2.0 license. See the LICENSE file for more info.




[travis-badge]: https://travis-ci.org/nathansizemore/simple-stream
[docs]: https://nathansizemore.github.io/simple-stream/simple_stream/index.html
[rust-openssl-repo]: https://github.com/sfackler/rust-openssl
