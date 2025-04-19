mod bait;
pub mod constants;
mod player;

use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
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
                            let plr = self.create_player(addr).await;
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
                                self.socket.send_to(msg_enemy_name.clone().as_bytes(), i).await.unwrap();
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
    async fn create_player(&self, addr: SocketAddr) -> Player {
        let player_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        println!("New player created: {}", player_id);

        // Create a new snake
        let player_snake = Snake::new(
            SNAKE_INITIAL_LENGTH as i32,
            rand::random_range(0..SNAKE_SKIN_COLOR_RANGE),
            SNAKE_SPEED,
        );

        // Create the player
        let new_player = Player::new(player_id, "Unnamed".to_string(), player_snake.clone(), addr);

        // Send first snake back to the client
        let mut msg = format!("{}1,", COMM_START_NEW_MESS);
        for (i, node) in player_snake.nodes.iter().enumerate() {
            msg.push_str(&format!("{:.4},{:.4}", node.x, node.y));
            if i < player_snake.nodes.len() - 1 {
                msg.push(',');
            }
        }

        self.socket.send_to(msg.as_bytes(), addr).await.unwrap();

        // Prepare new enemy message for other players
        let new_enemy_msg = format!("{}5,{},Unnamed,", COMM_START_NEW_MESS, player_id);

        let mut full_enemy_msg = new_enemy_msg;
        for (i, node) in player_snake.nodes.iter().enumerate() {
            full_enemy_msg.push_str(&format!("{:.4},{:.4}", node.x, node.y));
            if i < player_snake.nodes.len() - 1 {
                full_enemy_msg.push(',');
            }
        }

        // Send all other players to this new player
        let mut data = String::new();

        for other_player in self.players.lock().await.values() {
            if other_player.addr != new_player.addr {
                data.push_str(&format!(
                    "{}{}{}",
                    COMM_START_NEW_MESS, COMM_NEW_ENEMY, other_player.id
                ));
                data.push_str(&format!(",{},", other_player.name));

                for (j, node) in other_player.snake.nodes.iter().enumerate() {
                    data.push_str(&format!("{},{}", node.x, node.y));
                    if j < other_player.snake.nodes.len() - 1 {
                        data.push(',');
                    }
                }
            }
        }

        if !data.is_empty() {
            self.socket.send_to(data.as_bytes(), addr).await.unwrap();
        }

        // Send new player to all other players
        for other_player in self.players.lock().await.values() {
            if other_player.addr != new_player.addr {
                self.socket.send_to(full_enemy_msg.clone().as_bytes(), addr).await.unwrap();
            }
        }

        // Send all baits to the new player
        let bait = self.baits.lock().await;
        for bait_info in bait.iter() {
            let bait_msg = format!(
                "{}3,{},{},{}",
                COMM_START_NEW_MESS, bait_info.x, bait_info.y, bait_info.size
            );
            
            self.socket.send_to(bait_msg.as_bytes(), addr).await.unwrap();

        }

        println!("Total player(s): {}", self.players.lock().await.len());
        new_player
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
