mod bait;
pub mod constants;
mod player;

use std::{collections::HashMap, io::Read, net::SocketAddr, sync::Arc};
use tokio::{
    net::UdpSocket,
    sync::Mutex,
    time::{self, Duration},
};

use crate::bait::Bait;
use crate::player::{Player, snake::Snake};
use constants::*;

/// Main game server struct
struct GameServer {
    socket: Arc<UdpSocket>,
    players: Arc<Mutex<HashMap<SocketAddr, Player>>>,
    baits: Arc<Mutex<Vec<Bait>>>,
}

impl GameServer {
    /// Create a new GameServer bound to the given address
    async fn new(bind_addr: &str) -> tokio::io::Result<Self> {
        let socket = UdpSocket::bind(bind_addr).await?;
        Ok(Self {
            socket: Arc::new(socket),
            players: Arc::new(Mutex::new(HashMap::new())),
            baits: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Spawn the listening task that receives client packets
    pub fn start_listener(self: Arc<Self>) {
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
                            let plr = self.create_player(addr, data).await;
                            // Player::new(name, snake, addr)
                            println!("New player from {}", addr);
                            players_lock.insert(addr, plr);
                            
                        } else {
                            // Existing player
                            self.handle_command(addr, data).await;
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to receive packet: {}", e);
                    }
                }
            }
        });
    }

    /// Handle incoming commands from existing players
    async fn handle_command(&self, addr: SocketAddr, data: &[u8]) {
        let message = String::from_utf8_lossy(data);
        let splitted: Vec<&str> = message.split(',').collect();

        if splitted.is_empty() {
            return;
        }

        println!("{}", message);
        let mut lk = self.players.lock().await;

        // Try to find the player by address
        let player_id_opt = lk.get_mut(&addr);
        // println!("{}", player_id_opt.unwrap_or(0));

        match splitted[0] {
            "2" => {
                // Update player's mouse position
                if let Some(player_id) = player_id_opt {
                    if splitted.len() >= 5 {
                        player_id.update_xy(
                            splitted[1].parse().unwrap_or(0.0),
                            splitted[2].parse().unwrap_or(0.0),
                            splitted[3].parse().unwrap_or(0),
                            splitted[4].parse().unwrap_or(0),
                        );
                    }
                }
            }
            "9" => {
                // Player sends their name to all other players
                if let Some(player) = player_id_opt {
                    if splitted.len() >= 2 {
                        let name = splitted[1].to_string();

                        // Update the player's name
                        player.update_player_name(name.clone());

                        // Notify all other players
                        let msg_enemy_name =
                            format!("{}{}{}", COMM_START_NEW_MESS, COMM_ENEMY_NAME, player.id,);

                        for &i in self.players.lock().await.keys() {
                            if i != player.addr {
                                self.socket.send_to(msg_enemy_name.clone().as_bytes(), i);
                            }
                        }
                    }
                }
            }
            "10" => {
                // Player is accelerating
                if let Some(player_id) = player_id_opt {
                    player_id.update_player_acceleration(true);
                };
            }
            "11" => {
                // Player stops accelerating
                if let Some(player_id) = player_id_opt {
                    player_id.update_player_acceleration(false);
                };
            }
            _ => {
                println!("Inrecognized Command");
            }
        }
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

    /// Handle creation of a new player
    async fn create_player(&self, addr: SocketAddr, data: &[u8])-> Player {
        // Parse initial data, send welcome or initial state
        println!("Creating player {}: {:?}", addr, data);
        let welcome = b"Welcome to the game!";
        if let Err(e) = self.socket.send_to(welcome, addr).await {
            eprintln!("Failed to send welcome to {}: {}", addr, e);
        }
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
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl-c");
    println!("Shutting down server...");

    // Stop the game loop
    game_handle.abort();

    Ok(())
}
