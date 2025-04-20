use std::io;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::net::{UdpSocket, ToSocketAddrs};

/// UDP 收发包统计信息
#[derive(Debug)]
pub struct UdpStats {
    sent_packets: AtomicUsize,
    sent_bytes: AtomicUsize,
    received_packets: AtomicUsize,
    received_bytes: AtomicUsize,
}

impl Default for UdpStats {
    fn default() -> Self {
        Self {
            sent_packets: AtomicUsize::new(0),
            sent_bytes: AtomicUsize::new(0),
            received_packets: AtomicUsize::new(0),
            received_bytes: AtomicUsize::new(0),
        }
    }
}

impl UdpStats {
    pub fn sent_packets(&self) -> usize {
        self.sent_packets.load(Ordering::Relaxed)
    }

    pub fn sent_bytes(&self) -> usize {
        self.sent_bytes.load(Ordering::Relaxed)
    }

    pub fn received_packets(&self) -> usize {
        self.received_packets.load(Ordering::Relaxed)
    }

    pub fn received_bytes(&self) -> usize {
        self.received_bytes.load(Ordering::Relaxed)
    }
}

/// 封装了统计功能的 UDP Socket
pub struct MyUdpSocket {
    socket: UdpSocket,
    stats: UdpStats,
}

impl MyUdpSocket {
    /// 绑定到指定地址，创建 UDP Socket
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        Ok(Self {
            socket,
            stats: UdpStats::default(),
        })
    }

    /// 接收数据并记录统计信息
    pub async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        let result = self.socket.recv_from(buf).await;
        if let Ok((size, _)) = &result {
            self.stats.received_packets.fetch_add(1, Ordering::Relaxed);
            self.stats.received_bytes.fetch_add(*size, Ordering::Relaxed);
        }
        result
    }

    /// 发送数据并记录统计信息
    pub async fn send_to<A: ToSocketAddrs>(&self, buf: &[u8], addr: A) -> io::Result<usize> {
        let result = self.socket.send_to(buf, addr).await;
        if let Ok(size) = &result {
            self.stats.sent_packets.fetch_add(1, Ordering::Relaxed);
            self.stats.sent_bytes.fetch_add(*size, Ordering::Relaxed);
        }
        result
    }

    /// 获取统计信息引用
    pub fn stats(&self) -> &UdpStats {
        &self.stats
    }
}