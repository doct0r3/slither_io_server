mod player;
mod bait;

use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{net::UdpSocket, sync::Mutex, time::{self, Duration}};

use crate::player::{Player,snake::Snake};
use crate::bait::Bait;


/// Main game server struct
struct GameServer {
    socket: Arc<UdpSocket>,
    players: Arc<Mutex<HashMap<SocketAddr, Player>>>,
}

impl GameServer {
    /// Create a new GameServer bound to the given address
    async fn new(bind_addr: &str) -> tokio::io::Result<Self> {
        let socket = UdpSocket::bind(bind_addr).await?;
        Ok(Self {
            socket: Arc::new(socket),
            players: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Spawn the listening task that receives client packets
    fn start_listener(self: Arc<Self>) {
        let socket = Arc::clone(&self.socket);
        let players = Arc::clone(&self.players);

        tokio::spawn(async move {
            let mut buf = vec![0u8; 1024];
            loop {
                match socket.recv_from(&mut buf).await {
                    Ok((len, addr)) => {
                        let data = &buf[..len];
                        let mut players_lock = players.lock().await;
                        if !players_lock.contains_key(&addr) {
                            // New player
                            println!("New player from {}", addr);
                            let player = Player { addr };
                            players_lock.insert(addr, player);
                            create_player(&socket, addr, data).await;
                        } else {
                            // Existing player
                            handle_command(&socket, addr, data).await;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to receive packet: {}", e);
                    }
                }
            }
        });
    }

    /// Game loop: sends updates to all clients every 10 ms
    async fn game_loop(self: Arc<Self>) {
        let mut interval = time::interval(Duration::from_millis(10));
        loop {
            interval.tick().await;
            let players_lock = self.players.lock().await;
            for player in players_lock.values() {
                // Prepare your game state packet here
                let packet = b"game state update";
                if let Err(e) = self.socket.send_to(packet, player.addr).await {
                    eprintln!("Failed to send to {}: {}", player.addr, e);
                }
            }
        }
    }
}

/// Handle creation of a new player
async fn create_player(socket: &UdpSocket, addr: SocketAddr, data: &[u8]) {
    // Parse initial data, send welcome or initial state
    println!("Creating player {}: {:?}", addr, data);
    let welcome = b"Welcome to the game!";
    if let Err(e) = socket.send_to(welcome, addr).await {
        eprintln!("Failed to send welcome to {}: {}", addr, e);
    }
}

/// Handle incoming commands from existing players
async fn handle_command(socket: &UdpSocket, addr: SocketAddr, data: &[u8]) {
    // Parse command and update player state
    println!("Received command from {}: {:?}", addr, data);
    // Echo back or process accordingly
    if let Err(e) = socket.send_to(b"Command received", addr).await {
        eprintln!("Failed to acknowledge {}: {}", addr, e);
    }
}

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    // Bind UDP socket and create the game server
    let server = Arc::new(GameServer::new("0.0.0.0:8080").await?);

    // Start the listener task
    server.clone().start_listener();

    // Start the game loop in background
    let game_handle = tokio::spawn(server.clone().game_loop());

    // Wait for Ctrl-C to shut down
    tokio::signal::ctrl_c().await.expect("failed to listen for ctrl-c");
    println!("Shutting down server...");

    // Stop the game loop
    game_handle.abort();

    Ok(())
}
