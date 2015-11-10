// Copyright 2015 Nathan Sizemore <nathanrsizemore@gmail.com>
//
// This Source Code Form is subject to the
// terms of the Mozilla Public License, v.
// 2.0. If a copy of the MPL was not
// distributed with this file, You can
// obtain one at
// http://mozilla.org/MPL/2.0/.

use std::fmt;


#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FnctlError {
    /// Operation is prohibited by locks held by other processes.
    EAGAIN,
    /// fd is not an open file descriptor, or the command was F_SETLK or F_SETLKW and the
    /// file descriptor open mode doesn't match with the type of lock requested.
    EBADF,
    /// It was detected that the specified F_SETLKW command would cause a deadlock.
    EDEADLK,
    /// Lock is outside your accessible address space.
    EFAULT,
    /// For F_SETLKW, the command was interrupted by a signal; see signal(7).
    /// For F_GETLK and F_SETLK, the command was interrupted by a signal before the lock
    /// was checked or acquired. Most likely when locking a remote
    /// file (e.g., locking over NFS), but can sometimes happen locally.
    EINTR,
    /// For F_DUPFD, arg is negative or is greater than the maximum allowable value.
    /// For F_SETSIG, arg is not an allowable signal number.
    EINVAL,
    /// For F_DUPFD, the process already has the maximum number of file descriptors open.
    EMFILE,
    /// Too many segment locks open, lock table is full, or a remote locking protocol
    /// failed (e.g., locking over NFS).
    ENOLCK,
    /// Attempted to clear the O_APPEND flag on a file that has the append-only attribute set.
    EPERM
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ReadError {
    /// System is out of memory
    ENOMEM,
    /// The file descriptor fd refers to a file other than a socket and has been
    /// marked nonblocking (O_NONBLOCK), and the read would block.
    EAGAIN,
    /// fd is not a valid file descriptor or is not open for reading.
    EBADF,
    /// buf is outside your accessible address space.
    EFAULT,
    /// The call was interrupted by a signal before any data was read; see signal(7).
    EINTR,
    /// fd is attached to an object which is unsuitable for reading; or the file was opened
    /// with the O_DIRECT flag, and either the address specified in buf, the value specified
    /// in count, or the current file offset is not suitably aligned.
    /// OR
    /// fd was created via a call to timerfd_create(2) and the wrong size buffer was given to
    /// read(); see timerfd_create(2) for further information.
    EINVAL,
    /// I/O error. This will happen for example when the process is in a background process
    /// group, tries to read from its controlling terminal, and either it is ignoring or
    /// blocking SIGTTIN or its process group is orphaned. It may also occur when there
    /// is a low-level I/O error while reading from a disk or tape.
    EIO,
    /// fd refers to a directory.
    EISDIR,
    /// End of file has been reached. No more can be read from this socket.
    EOF,
    /// fd is connected to a TCP socket where the other end has been closed, but the write
    /// command has not had enough time to finish writing to the internal buffer
    ECONNRESET
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum WriteError {
    /// The file descriptor fd refers to a file other than a socket and has been
    /// marked nonblocking (O_NONBLOCK), and the write would block.
    EAGAIN,
    /// The file descriptor fd refers to a socket and has been marked nonblocking (O_NONBLOCK),
    /// and the write would block. POSIX.1-2001 allows either error to be returned for this case,
    /// and does not require these constants to have the same value, so a portable application
    /// should check for both possibilities.
    EWOULDBLOCK,
    /// fd is not a valid file descriptor or is not open for writing.
    EBADF,
    /// fd refers to a datagram socket for which a peer address has not been set using connect(2).
    EDESTADDRREQ,
    /// The user's quota of disk blocks on the file system containing the file referred
    /// to by fd has been exhausted.
    EDQUOT,
    /// buf is outside your accessible address space.
    EFAULT,
    /// An attempt was made to write a file that exceeds the implementation-defined maximum
    /// file size or the process's file size limit, or to write at a position past the
    /// maximum allowed offset.
    EFBIG,
    /// The call was interrupted by a signal before any data was written; see signal(7).
    EINTR,
    /// fd is attached to an object which is unsuitable for writing; or the file was opened
    /// with the O_DIRECT flag, and either the address specified in buf, the value specified
    /// in count, or the current file offset is not suitably aligned.
    EINVAL,
    /// A low-level I/O error occurred while modifying the inode.
    EIO,
    /// The device containing the file referred to by fd has no room for the data.
    ENOSPC,
    /// fd is connected to a pipe or socket whose reading end is closed. When this happens
    /// the writing process will also receive a SIGPIPE signal. (Thus, the write return value
    /// is seen only if the program catches, blocks or ignores this signal.)
    EPIPE,
    /// fd is connected to a TCP socket where the other end has been closed, but the write
    /// command has not had enough time to finish writing to the internal buffer
    ECONNRESET
}

impl fmt::Display for FnctlError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FnctlError::EAGAIN => "EAGAIN".fmt(f),
            FnctlError::EBADF => "EBADF".fmt(f),
            FnctlError::EDEADLK => "EDEADLK".fmt(f),
            FnctlError::EFAULT => "EFAULT".fmt(f),
            FnctlError::EINTR => "EINTR".fmt(f),
            FnctlError::EINVAL => "EINVAL".fmt(f),
            FnctlError::EMFILE => "EMFILE".fmt(f),
            FnctlError::ENOLCK => "ENOLCK".fmt(f),
            FnctlError::EPERM => "EPERM".fmt(f),
        }
    }
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReadError::ENOMEM => "ENOMEM".fmt(f),
            ReadError::EAGAIN => "EAGAIN".fmt(f),
            ReadError::EBADF => "EBADF".fmt(f),
            ReadError::EFAULT => "EFAULT".fmt(f),
            ReadError::EINTR => "EINTR".fmt(f),
            ReadError::EINVAL => "EINVAL".fmt(f),
            ReadError::EIO => "EIO".fmt(f),
            ReadError::EISDIR => "EISDIR".fmt(f),
            ReadError::EOF => "EOF".fmt(f),
            ReadError::ECONNRESET => "ECONNRESET".fmt(f)
        }
    }
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WriteError::EAGAIN => "EAGAIN".fmt(f),
            WriteError::EWOULDBLOCK => "EWOULDBLOCK".fmt(f),
            WriteError::EBADF => "EBADF".fmt(f),
            WriteError::EDESTADDRREQ => "EDESTADDRREQ".fmt(f),
            WriteError::EDQUOT => "EDQUOT".fmt(f),
            WriteError::EFAULT => "EFAULT".fmt(f),
            WriteError::EFBIG => "EFBIG".fmt(f),
            WriteError::EINTR => "EINTR".fmt(f),
            WriteError::EINVAL => "EINVAL".fmt(f),
            WriteError::EIO => "EIO".fmt(f),
            WriteError::ENOSPC => "ENOSPC".fmt(f),
            WriteError::EPIPE => "EPIPE".fmt(f),
            WriteError::ECONNRESET => "ECONNRESET".fmt(f)
        }
    }
}
