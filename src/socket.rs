// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the terms of the
// Mozilla Public License, v. 2.0. If a copy of the MPL was not
// distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.


use std::mem;
use std::ffi::CString;
use std::os::unix::io::{RawFd, AsRawFd};
use std::io::{Read, Write, Error, ErrorKind};

use libc;
use errno::errno;
use libc::{c_int, c_void};

use stream::StreamShutdown;


/// The `TcpOptions` trait allows for various TCP level settings.
pub trait TcpOptions {
    /// If set, disable the Nagle algorithm. This means that segments are always sent as soon as
    /// possible, even if there is only a small amount of data. When not set, data is buffered
    /// until there is a sufficient amount to send out, thereby avoiding the frequent sending of
    /// small packets, which results in poor utilization of the network. This option is
    /// overridden by TCP_CORK; however, setting this option forces an explicit flush of pending
    /// output, even if TCP_CORK is currently set.
    fn set_tcp_nodelay(&mut self, nodelay: bool) -> Result<(), Error>;
}

/// The `SocketOptions` trait allows for various socket level settings.
pub trait SocketOptions {
    /// Bind this socket to a particular device like "eth0", as specified in the passed
    /// interface name. If the name is an empty string or the option length is zero, the socket
    /// device binding is removed. The passed option is a variable-length null-terminated
    /// interface name string with the maximum size of IFNAMSIZ. If a socket is bound to an
    /// interface, only packets received from that particular interface are processed by the
    /// socket. Note that this only works for some socket types, particularly AF_INET sockets.
    /// It is not supported for packet sockets (use normal bind(2) there).
    ///
    /// Before Linux 3.8, this socket option could be set, but could not retrieved with
    /// getsockopt(2). Since Linux 3.8, it is readable. The optlen argument should contain the
    /// buffer size available to receive the device name and is recommended to be IFNAMSZ bytes.
    /// The real device name length is reported back in the optlen argument.
    fn set_bindtodevice(&mut self, interface: String) -> Result<(), Error>;
    /// When enabled, datagram sockets are allowed to send packets to a broadcast address.
    /// This option has no effect on stream-oriented sockets.
    fn set_broadcast(&mut self, option: bool) -> Result<(), Error>;
    /// Enable BSD bug-to-bug compatibility. This is used by the UDP protocol module in
    /// Linux 2.0 and 2.2. If enabled ICMP errors received for a UDP socket will not be passed
    /// to the user program. In later kernel versions, support for this option has been phased
    /// out: Linux 2.4 silently ignores it, and Linux 2.6 generates a kernel warning
    /// (printk()) if a program uses this option. Linux 2.0 also enabled BSD bug-to-bug
    /// compatibility options (random header changing, skipping of the broadcast flag) for raw
    /// sockets with this option, but that was removed in Linux 2.2.
    fn set_bsdcompat(&mut self, option: bool) -> Result<(), Error>;
    /// Enable socket debugging. Only allowed for processes with the CAP_NET_ADMIN capability
    /// or an effective user ID of 0.
    fn set_debug(&mut self, option: bool) -> Result<(), Error>;
    /// Don't send via a gateway, only send to directly connected hosts. The same effect can be
    /// achieved by setting the MSG_DONTROUTE flag on a socket send(2) operation. Expects an
    /// integer boolean flag.
    fn set_dontroute(&mut self, option: bool) -> Result<(), Error>;
    /// Enable sending of keep-alive messages on connection-oriented sockets. Expects an integer
    /// boolean flag.
    fn set_keepalive(&mut self, option: bool) -> Result<(), Error>;
    /// Sets or gets the SO_LINGER option. When enabled, a close(2) or shutdown(2) will not return
    /// until all queued messages for the socket have been successfully sent or the linger timeout
    /// has been reached. Otherwise, the call returns immediately and the closing is done in the
    /// background. When the socket is closed as part of exit(2), it always lingers in the
    /// background.
    fn set_linger(&mut self, option: bool, sec: u32) -> Result<(), Error>;
    /// Set the mark for each packet sent through this socket (similar to the netfilter MARK
    /// target but socket-based). Changing the mark can be used for mark-based routing without
    /// netfilter or for packet filtering. Setting this option requires the CAP_NET_ADMIN
    /// capability.
    fn set_mark(&mut self, option: bool) -> Result<(), Error>;
    /// If this option is enabled, out-of-band data is directly placed into the receive data
    /// stream. Otherwise out-of-band data is only passed when the MSG_OOB flag is set during
    /// receiving.
    fn set_oobinline(&mut self, option: bool) -> Result<(), Error>;
    /// Enable or disable the receiving of the SCM_CREDENTIALS control message. For more
    /// information see unix(7).
    fn set_passcred(&mut self, option: bool) -> Result<(), Error>;
    /// Set the protocol-defined priority for all packets to be sent on this socket. Linux uses
    /// this value to order the networking queues: packets with a higher priority may be processed
    /// first depending on the selected device queueing discipline. For ip(7), this also sets the
    /// IP type-of-service (TOS) field for outgoing packets. Setting a priority outside the
    /// range 0 to 6 requires the CAP_NET_ADMIN capability.
    fn set_priority(&mut self, priority: u32) -> Result<(), Error>;
    /// Sets or gets the maximum socket receive buffer in bytes. The kernel doubles this value
    /// (to allow space for bookkeeping overhead) when it is set using setsockopt(2), and this
    /// doubled value is returned by getsockopt(2). The default value is set by
    /// the /proc/sys/net/core/rmem_default file, and the maximum allowed value is set by
    /// the /proc/sys/net/core/rmem_max file. The minimum (doubled) value for this option is 256.
    fn set_rcvbuf(&mut self, size: usize) -> Result<(), Error>;
    /// Using this socket option, a privileged (CAP_NET_ADMIN) process can perform the same task
    /// as SO_RCVBUF, but the rmem_max limit can be overridden.
    fn set_rcvbufforce(&mut self, size: usize) -> Result<(), Error>;
    /// Specify the minimum number of bytes in the buffer until the socket layer will pass the
    /// data to the protocol (SO_SNDLOWAT) or the user on receiving (SO_RCVLOWAT). These two
    /// values are initialized to 1. SO_SNDLOWAT is not changeable on Linux (setsockopt(2) fails
    /// with the error ENOPROTOOPT). SO_RCVLOWAT is changeable only since Linux 2.4. The select(2)
    /// and poll(2) system calls currently do not respect the SO_RCVLOWAT setting on Linux, and
    /// mark a socket readable when even a single byte of data is available. A subsequent read
    /// from the socket will block until SO_RCVLOWAT bytes are available.
    fn set_rcvlowat(&mut self, bytes: usize) -> Result<(), Error>;
    /// Specify the minimum number of bytes in the buffer until the socket layer will pass the
    /// data to the protocol (SO_SNDLOWAT) or the user on receiving (SO_RCVLOWAT). These two
    /// values are initialized to 1. SO_SNDLOWAT is not changeable on Linux (setsockopt(2) fails
    /// with the error ENOPROTOOPT). SO_RCVLOWAT is changeable only since Linux 2.4. The select(2)
    /// and poll(2) system calls currently do not respect the SO_RCVLOWAT setting on Linux, and
    /// mark a socket readable when even a single byte of data is available. A subsequent read
    /// from the socket will block until SO_RCVLOWAT bytes are available.
    fn set_sndlowat(&mut self, bytes: usize) -> Result<(), Error>;
    /// Specify the receiving or sending timeouts until reporting an error. The argument is a
    /// struct timeval. If an input or output function blocks for this period of time, and data
    /// has been sent or received, the return value of that function will be the amount of data
    /// transferred; if no data has been transferred and the timeout has been reached then -1 is
    /// returned with errno set to EAGAIN or EWOULDBLOCK, or EINPROGRESS (for connect(2)) just as
    /// if the socket was specified to be nonblocking. If the timeout is set to zero (the default)
    /// then the operation will never timeout. Timeouts only have effect for system calls that
    /// perform socket I/O (e.g., read(2), recvmsg(2), send(2), sendmsg(2)); timeouts have no
    /// effect for select(2), poll(2), epoll_wait(2), and so on.
    fn set_rcvtimeo(&mut self,
                    sec: libc::time_t,
                    micro_sec: libc::suseconds_t)
                    -> Result<(), Error>;
    /// Specify the receiving or sending timeouts until reporting an error. The argument is a
    /// struct timeval. If an input or output function blocks for this period of time, and data
    /// has been sent or received, the return value of that function will be the amount of data
    /// transferred; if no data has been transferred and the timeout has been reached then -1 is
    /// returned with errno set to EAGAIN or EWOULDBLOCK, or EINPROGRESS (for connect(2)) just as
    /// if the socket was specified to be nonblocking. If the timeout is set to zero (the default)
    /// then the operation will never timeout. Timeouts only have effect for system calls that
    /// perform socket I/O (e.g., read(2), recvmsg(2), send(2), sendmsg(2)); timeouts have no
    /// effect for select(2), poll(2), epoll_wait(2), and so on.
    fn set_sndtimeo(&mut self,
                    sec: libc::time_t,
                    micro_sec: libc::suseconds_t)
                    -> Result<(), Error>;
    /// Indicates that the rules used in validating addresses supplied in a bind(2) call should
    /// allow reuse of local addresses. For AF_INET sockets this means that a socket may bind,
    /// except when there is an active listening socket bound to the address. When the listening
    /// socket is bound to INADDR_ANY with a specific port then it is not possible to bind to this
    /// port for any local address. Argument is an integer boolean flag.
    fn set_reuseaddr(&mut self, option: bool) -> Result<(), Error>;
    /// Sets or gets the maximum socket send buffer in bytes. The kernel doubles this value
    /// (to allow space for bookkeeping overhead) when it is set using setsockopt(2), and this
    /// doubled value is returned by getsockopt(2). The default value is set by
    /// the /proc/sys/net/core/wmem_default file and the maximum allowed value is set by
    /// the /proc/sys/net/core/wmem_max file. The minimum (doubled) value for this option is 2048.
    fn set_sndbuf(&mut self, size: usize) -> Result<(), Error>;
    /// Using this socket option, a privileged (CAP_NET_ADMIN) process can perform the same task
    /// as SO_SNDBUF, but the wmem_max limit can be overridden.
    fn set_sndbufforce(&mut self, size: usize) -> Result<(), Error>;
    /// Enable or disable the receiving of the SO_TIMESTAMP control message. The timestamp control
    /// message is sent with level SOL_SOCKET and the cmsg_data field is a struct timeval
    /// indicating the reception time of the last packet passed to the user in this call.
    /// See cmsg(3) for details on control messages.
    fn set_timestamp(&mut self, option: bool) -> Result<(), Error>;
    /// Sets the `O_NONBLOCK` flag on the underlying fd
    fn set_nonblocking(&mut self) -> Result<(), Error>;
}


#[derive(Clone, Eq, PartialEq, Debug)]
/// Wrapper for file descriptor based sockets
pub struct Socket {
    fd: RawFd,
}

impl Socket {
    /// Creates a new socket with assumed ownership of `fd`
    pub fn new(fd: RawFd) -> Socket {
        Socket {
            fd: fd
        }
    }
}

impl TcpOptions for Socket {
    fn set_tcp_nodelay(&mut self, nodelay: bool) -> Result<(), Error> {
        let optval: c_int = match nodelay {
            true => 1,
            false => 0
        };
        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             libc::IPPROTO_TCP,
                             libc::TCP_NODELAY,
                             &optval as *const _ as *const c_void,
                             mem::size_of::<c_int>() as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }
}

impl SocketOptions for Socket {
    fn set_nonblocking(&mut self) -> Result<(), Error> {
        let result = unsafe {
            libc::fcntl(self.as_raw_fd(), libc::F_GETFL, 0)
        };
        if result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        let flags = result | libc::O_NONBLOCK;
        let result = unsafe {
            libc::fcntl(self.as_raw_fd(), libc::F_SETFL, flags)
        };
        if result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    fn set_bindtodevice(&mut self, interface: String) -> Result<(), Error> {
        const SO_BINDTODEVICE: i32 = 25;
        let cstr_result = CString::new(interface);
        if cstr_result.is_err() {
            return Err(Error::new(ErrorKind::Other, "Null Byte"));
        }

        let cstr = cstr_result.unwrap();
        unsafe {
            if libc::strlen(cstr.as_ptr()) > libc::IF_NAMESIZE {
                return Err(Error::new(ErrorKind::Other, "strlen(interface) > IFNAMSIZ"));
            }
        }

        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             libc::SOL_SOCKET,
                             SO_BINDTODEVICE,
                             cstr.as_ptr() as *const c_void,
                             libc::strlen(cstr.as_ptr()) as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    fn set_broadcast(&mut self, option: bool) -> Result<(), Error> {
        let optval: c_int = match option {
            true => 1,
            false => 0
        };
        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             libc::SOL_SOCKET,
                             libc::SO_BROADCAST,
                             &optval as *const _ as *const c_void,
                             mem::size_of::<c_int>() as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    fn set_bsdcompat(&mut self, option: bool) -> Result<(), Error> {
        const SO_BSDCOMPAT: i32 = 14;
        let optval: c_int = match option {
            true => 1,
            false => 0
        };
        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             libc::SOL_SOCKET,
                             SO_BSDCOMPAT,
                             &optval as *const _ as *const c_void,
                             mem::size_of::<c_int>() as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    fn set_debug(&mut self, option: bool) -> Result<(), Error> {
        let optval: c_int = match option {
            true => 1,
            false => 0
        };
        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             libc::SOL_SOCKET,
                             libc::SO_DEBUG,
                             &optval as *const _ as *const c_void,
                             mem::size_of::<c_int>() as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    fn set_dontroute(&mut self, option: bool) -> Result<(), Error> {
        let optval: c_int = match option {
            true => 1,
            false => 0
        };
        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             libc::SOL_SOCKET,
                             libc::SO_DONTROUTE,
                             &optval as *const _ as *const c_void,
                             mem::size_of::<c_int>() as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    fn set_keepalive(&mut self, option: bool) -> Result<(), Error> {
        let optval: c_int = match option {
            true => 1,
            false => 0
        };
        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             libc::SOL_SOCKET,
                             libc::SO_KEEPALIVE,
                             &optval as *const _ as *const c_void,
                             mem::size_of::<c_int>() as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    fn set_linger(&mut self, option: bool, sec: u32) -> Result<(), Error> {
        #[repr(C, packed)]
        struct Linger {
            l_onoff: c_int,
            l_linger: c_int
        };

        let optval: c_int = match option {
            true => 1,
            false => 0
        };
        let data = Linger {
            l_onoff: optval,
            l_linger: sec as i32
        };

        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             libc::SOL_SOCKET,
                             libc::SO_LINGER,
                             &data as *const _ as *const c_void,
                             mem::size_of::<Linger>() as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    fn set_mark(&mut self, option: bool) -> Result<(), Error> {
        const SO_MARK: i32 = 36;
        let optval: c_int = match option {
            true => 1,
            false => 0
        };
        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             libc::SOL_SOCKET,
                             SO_MARK,
                             &optval as *const _ as *const c_void,
                             mem::size_of::<c_int>() as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    fn set_oobinline(&mut self, option: bool) -> Result<(), Error> {
        let optval: c_int = match option {
            true => 1,
            false => 0
        };
        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             libc::SOL_SOCKET,
                             libc::SO_OOBINLINE,
                             &optval as *const _ as *const c_void,
                             mem::size_of::<c_int>() as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    fn set_passcred(&mut self, option: bool) -> Result<(), Error> {
        const SO_PASSCRED: i32 = 16;
        let optval: c_int = match option {
            true => 1,
            false => 0
        };
        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             libc::SOL_SOCKET,
                             SO_PASSCRED,
                             &optval as *const _ as *const c_void,
                             mem::size_of::<c_int>() as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    fn set_priority(&mut self, priority: u32) -> Result<(), Error> {
        const SO_PRIORITY: i32 = 12;
        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             libc::SOL_SOCKET,
                             SO_PRIORITY,
                             &priority as *const _ as *const c_void,
                             mem::size_of::<u32>() as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    fn set_rcvbuf(&mut self, size: usize) -> Result<(), Error> {
        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             libc::SOL_SOCKET,
                             libc::SO_RCVBUF,
                             &size as *const _ as *const c_void,
                             mem::size_of::<usize>() as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    fn set_rcvbufforce(&mut self, size: usize) -> Result<(), Error> {
        self.set_rcvbuf(size)
    }

    fn set_rcvlowat(&mut self, bytes: usize) -> Result<(), Error> {
        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             libc::SOL_SOCKET,
                             libc::SO_RCVLOWAT,
                             &bytes as *const _ as *const c_void,
                             mem::size_of::<usize>() as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    fn set_sndlowat(&mut self, bytes: usize) -> Result<(), Error> {
        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             libc::SOL_SOCKET,
                             libc::SO_SNDLOWAT,
                             &bytes as *const _ as *const c_void,
                             mem::size_of::<usize>() as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    #[cfg(target_arch = "x86")]
    fn set_rcvtimeo(&mut self, sec: i32, micro_sec: i32) -> Result<(), Error> {
        #[repr(C, packed)]
        struct Timeval {
            tv_sec: libc::time_t,
            tv_usec: libc::suseconds_t
        };
        let data = Timeval {
            tv_sec: sec,
            tv_usec: micro_sec
        };

        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             libc::SOL_SOCKET,
                             libc::SO_RCVTIMEO,
                             &data as *const _ as *const c_void,
                             mem::size_of::<Timeval>() as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    fn set_rcvtimeo(&mut self,
                    sec: libc::time_t,
                    micro_sec: libc::suseconds_t)
                    -> Result<(), Error>
    {
        #[repr(C, packed)]
        struct Timeval {
            tv_sec: libc::time_t,
            tv_usec: libc::suseconds_t
        };
        let data = Timeval {
            tv_sec: sec,
            tv_usec: micro_sec
        };

        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             libc::SOL_SOCKET,
                             libc::SO_RCVTIMEO,
                             &data as *const _ as *const c_void,
                             mem::size_of::<Timeval>() as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    fn set_sndtimeo(&mut self,
                    sec: libc::time_t,
                    micro_sec: libc::suseconds_t)
                    -> Result<(), Error>
    {
        #[repr(C, packed)]
        struct Timeval {
            tv_sec: libc::time_t,
            tv_usec: libc::suseconds_t
        };
        let data = Timeval {
            tv_sec: sec,
            tv_usec: micro_sec
        };

        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             libc::SOL_SOCKET,
                             libc::SO_SNDTIMEO,
                             &data as *const _ as *const c_void,
                             mem::size_of::<Timeval>() as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    fn set_reuseaddr(&mut self, option: bool) -> Result<(), Error> {
        let optval: c_int = match option {
            true => 1,
            false => 0
        };
        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             libc::SOL_SOCKET,
                             libc::SO_REUSEADDR,
                             &optval as *const _ as *const c_void,
                             mem::size_of::<c_int>() as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    fn set_sndbuf(&mut self, size: usize) -> Result<(), Error> {
        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             libc::SOL_SOCKET,
                             libc::SO_SNDBUF,
                             &size as *const _ as *const c_void,
                             mem::size_of::<usize>() as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }

    fn set_sndbufforce(&mut self, size: usize) -> Result<(), Error> {
        self.set_sndbuf(size)
    }

    fn set_timestamp(&mut self, option: bool) -> Result<(), Error> {
        const SO_TIMESTAMP: i32 = 29;
        let optval: c_int = match option {
            true => 1,
            false => 0
        };
        let opt_result = unsafe {
            libc::setsockopt(self.fd,
                             libc::SOL_SOCKET,
                             SO_TIMESTAMP,
                             &optval as *const _ as *const c_void,
                             mem::size_of::<c_int>() as u32)
        };
        if opt_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }
}

impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let result = unsafe {
            libc::read(self.fd, buf as *mut _ as *mut c_void, buf.len())
        };

        if result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        if result == 0 {
            return Err(Error::new(ErrorKind::UnexpectedEof, "UnexpectedEof"));
        }

        Ok(result as usize)
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        let result = unsafe {
            libc::write(self.fd, buf as *const _ as *const c_void, buf.len())
        };

        if result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(result as usize)
    }

    fn flush(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

impl StreamShutdown for Socket {
    fn shutdown(&mut self) -> Result<(), Error> {
        let shutdown_result = unsafe {
            libc::shutdown(self.fd, libc::SHUT_RDWR)
        };
        if shutdown_result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        let result = unsafe {
            libc::close(self.fd)
        };
        if result < 0 {
            return Err(Error::from_raw_os_error(errno().0 as i32));
        }

        Ok(())
    }
}

impl AsRawFd for Socket {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}
