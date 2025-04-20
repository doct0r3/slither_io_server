use std::{
    collections::HashMap,
    io,
    net::SocketAddr,
    sync::atomic::{AtomicUsize, Ordering},
    sync::Arc,
};
use tokio::{
    net::{ToSocketAddrs, UdpSocket},
    sync::Mutex,
};

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

/// A UDP socket that tracks per‐peer send/recv stats.
#[derive(Clone)]
pub struct MyUdpSocket {
    socket: Arc<UdpSocket>,
    stats: Arc<Mutex<HashMap<SocketAddr, UdpStats>>>,
}

impl MyUdpSocket {
    /// Bind once to a local address.
    pub async fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        Ok(Self {
            socket: Arc::new(socket),
            stats: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Receive a datagram and update the stats for the peer addr.
    pub async fn recv_from(&self, buf: &mut [u8]) -> io::Result<(usize, SocketAddr)> {
        let result = self.socket.recv_from(buf).await;
        if let Ok((size, peer)) = &result {
            let mut map = self.stats.lock().await;
            let entry = map.entry(*peer).or_insert_with(UdpStats::default);
            entry.received_packets.fetch_add(1, Ordering::Relaxed);
            entry.received_bytes.fetch_add(*size, Ordering::Relaxed);
        }
        result
    }

    /// Send a datagram to *this* peer and update its stats.
    pub async fn send_to(&self, buf: &[u8], peer: SocketAddr) -> io::Result<usize> {
        let result = self.socket.send_to(buf, peer).await;
        if let Ok(size) = &result {
            let mut map = self.stats.lock().await;
            let entry = map.entry(peer).or_insert_with(UdpStats::default);
            entry.sent_packets.fetch_add(1, Ordering::Relaxed);
            entry.sent_bytes.fetch_add(*size, Ordering::Relaxed);
        }
        result
    }

    /// Get a snapshot of all per‐peer stats as `(sent_pkts, sent_bytes, recv_pkts, recv_bytes)`.
    pub async fn get_stats(
        &self,
    ) -> HashMap<SocketAddr, (usize, usize, usize, usize)> {
        let map = self.stats.lock().await;
        map.iter()
            .map(|(peer, stat)| {
                (
                    *peer,
                    (
                        stat.sent_packets(),
                        stat.sent_bytes(),
                        stat.received_packets(),
                        stat.received_bytes(),
                    ),
                )
            })
            .collect()
    }

    /// Convenience: get stats for one specific peer, if any.
    pub async fn stats_for(
        &self,
        peer: &SocketAddr,
    ) -> Option<(usize, usize, usize, usize)> {
        let map = self.stats.lock().await;
        map.get(peer).map(|stat| {
            (
                stat.sent_packets(),
                stat.sent_bytes(),
                stat.received_packets(),
                stat.received_bytes(),
            )
        })
    }
}
