use std::fmt;
use std::io;
use std::net;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

use anyhow::Context as _;
use anyhow::Result;
use pin_project_lite::pin_project;
use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;
use tokio::io::ReadBuf;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::net::tcp;

use crate::config::Address;

/// # Socket Address Enum
///
/// Represents a network socket address. Currently supports IP-based addresses.
///
/// ## Variants
///
/// - `Ip(net::SocketAddr)`: Represents an IP socket address.
///
/// ## Example
///
/// ```rust
/// let addr = SocketAddr::Ip("127.0.0.1:8080".parse().unwrap()); 
/// ```
#[derive(Debug)]
pub enum SocketAddr {
    Ip(net::SocketAddr),
}

impl SocketAddr {
    // # Get Port Number
    ///
    /// Retrieves the port number from the socket address if available.
    ///
    /// ## Returns
    ///
    /// An `Option<u16>` containing the port number or `None` if not applicable.
    ///
    /// ## Example
    ///
    /// ```rust
    /// let addr = SocketAddr::Ip("127.0.0.1:8080".parse().unwrap());
    /// assert_eq!(addr.port(), Some(8080));
    /// ```
    pub fn port(&self) -> Option<u16> {
        match self {
            SocketAddr::Ip(addr) => Some(addr.port()),
        }
    }
}

impl Clone for SocketAddr {
    fn clone(&self) -> Self {
        match self {
            SocketAddr::Ip(addr) => SocketAddr::Ip(*addr),
        }
    }
}

impl fmt::Display for SocketAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SocketAddr::Ip(addr) => write!(f, "{}:{}", addr.ip(), addr.port()),
        }
    }
}

impl From<net::SocketAddr> for SocketAddr {
    fn from(val: net::SocketAddr) -> Self { SocketAddr::Ip(val) }
}

#[cfg(target_family = "unix")]
pin_project! {
    #[project = OwnedReadHalfProj]
    pub enum OwnedReadHalf {
        Tcp{#[pin] tcp: tcp::OwnedReadHalf},
    }
}

impl AsyncRead for OwnedReadHalf {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match self.project() {
            OwnedReadHalfProj::Tcp { tcp } => tcp.poll_read(cx, buf),
        }
    }
}

#[cfg(target_family = "unix")]
pin_project! {
    #[project = OwnedWriteHalfProj]
    pub enum OwnedWriteHalf {
        Tcp{#[pin] tcp: tcp::OwnedWriteHalf},
    }
}

impl AsyncWrite for OwnedWriteHalf {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        match self.project() {
            OwnedWriteHalfProj::Tcp { tcp } => tcp.poll_write(cx, buf),
        }
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[io::IoSlice<'_>],
    ) -> Poll<Result<usize, io::Error>> {
        match self.project() {
            OwnedWriteHalfProj::Tcp { tcp } => tcp.poll_write_vectored(cx, bufs),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.project() {
            OwnedWriteHalfProj::Tcp { tcp } => tcp.poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        match self.project() {
            OwnedWriteHalfProj::Tcp { tcp } => tcp.poll_shutdown(cx),
        }
    }
}

#[cfg(target_family = "unix")]
pin_project! {
    #[project = StreamProj]
    pub enum Stream {
        Tcp{#[pin] tcp: TcpStream},
    }
}

impl Stream {
    pub fn into_split(self) -> (OwnedReadHalf, OwnedWriteHalf) {
        match self {
            Stream::Tcp { tcp } => {
                let (read, write) = tcp.into_split();
                (OwnedReadHalf::Tcp { tcp: read }, OwnedWriteHalf::Tcp {
                    tcp: write,
                })
            }
        }
    }
}

pub enum Listener {
    Tcp(TcpListener),
}

impl Listener {
    pub async fn bind(addr: &Address) -> Result<Listener> {
        match addr {
            Address::Tcp(ip_addr, port) => TcpListener::bind((*ip_addr, *port))
                .await
                .with_context(|| format!("binding to tcp socket {ip_addr}:{port}"))
                .map(Listener::Tcp),
        }
    }

    pub async fn accept(&self) -> io::Result<(Stream, SocketAddr)> {
        match self {
            Listener::Tcp(tcp) => {
                let (stream, addr) = tcp.accept().await?;
                Ok((Stream::Tcp { tcp: stream }, addr.into()))
            }
        }
    }
}
