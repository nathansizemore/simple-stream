# simple-stream [<img src="https://travis-ci.org/nathansizemore/simple-stream.png?branch=master">](https://travis-ci.org/nathansizemore/simple-stream)
[Documentation](https://nathansizemore.github.io/simple-stream/simple_stream/index.html)

---

Crate providing various wrappers for [TcpStream](https://doc.rust-lang.org/stable/std/net/struct.TcpStream.html).

## Streams Currently Available

[NbetStream](https://nathansizemore.github.io/simple-stream/simple_stream/nbetstream/index.html)

[Bstream](https://nathansizemore.github.io/simple-stream/simple_stream/nbetstream/index.html)

## Streams Still Needed

###### SslNbetStream
###### SslBStream
###### WsNbetStream
###### WsBStream


## Data Framing

Streams read and write _messages_ based on a simple framing pattern:

~~~
0                   1                   2                   3
0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|          Payload Len        |           Payload           |
+-----------------------------------------------------------+
|                   Payload Data Continued                  |
+-----------------------------------------------------------+

Payload Len:    16 bits
Payload Data:   (Payload Len) bytes
~~~

## Usage

Add the following to your `Cargo.toml`

~~~toml
[dependencies.simple-stream]
git = "https://github.com/nathansizemore/simple-stream"
~~~

Add the following to your crate root

~~~rust
extern crate simple_stream;
~~~

## Author

Nathan Sizemore, nathanrsizemore@gmail.com

## License

simple-stream is available under the MPL-2.0 license. See the LICENSE file for more info.
