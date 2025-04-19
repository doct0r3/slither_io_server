mod bait;
mod collision;
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
    sync::{Mutex, MutexGuard},
    time::{self, Duration},
};

use crate::bait::Bait;
use crate::player::{Player, snake::Snake};
use collision::{Rect, rect_intersect};
use constants::*;

// Generate a random bait
pub fn generate_bait(low: f64, high: f64) -> Bait {
    let x = rand::random_range(low..high);
    let y = rand::random_range(low..high);

    let color = format!("{}", rand::random_range(0..MAX_BAIT_COLOR_RANGE));
    let size = rand::random_range(0.0..MAX_BAIT_SIZE as f64);
    Bait::new(x, y, color, size)
}

// Generate mass baits based on a dead snake
pub fn generate_mass_bait(snake: &Snake) -> Vec<bait::Bait> {
    let mut new_bait_arr = Vec::new();
    let color = rand::random_range(0..MAX_BAIT_COLOR_RANGE).to_string();

    for i in (0..snake.nodes.len()).step_by(2) {
        if i >= snake.nodes.len() - 1 {
            break;
        }

        let offset_x = rand::random_range(-5.0..5.0);
        let offset_y = rand::random_range(-5.0..5.0);

        let new_bait = Bait::new(
            snake.nodes[i].x + offset_x,
            snake.nodes[i].y + offset_y,
            color.clone(),
            MAX_BAITS_SIZE_ON_DEAD as f64,
        );

        new_bait_arr.push(new_bait);
    }

    new_bait_arr
}

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
                            // let lk1 = players_lock.clone();
                            // New player
                            self.create_player(addr, players_lock).await;
                            // Player::new(name, snake, addr)
                            println!("New player from {}", addr);
                        } else {
                            // let lk2 = players_lock.clone();

                            // Existing player
                            self.handle_command(addr, data, players_lock).await;
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
    async fn handle_command(&self, addr: SocketAddr, data: &[u8], mut players_lock: MutexGuard<'_, HashMap<SocketAddr, Player>>) {
        let message = String::from_utf8_lossy(data);
        let splitted: Vec<&str> = message.split(',').collect();

        if splitted.is_empty() {
            return;
        }

        println!("{}", message);
        let lk = players_lock.clone();

        // Try to find the player by address
        let player_id_opt = players_lock.get_mut(&addr);
        // println!("{}", player_id_opt.unwrap_or(0));

        match splitted[0] {
            "2" => {
                // Update player's mouse position
                if let Some(player_id) = player_id_opt {
                    if splitted.len() >= 5 {
                        if splitted[1].parse().unwrap_or(0.0) == 0.0 {
                            println!("RESET!!!");
                            
                        }
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

                        for &i in lk.keys() {
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
        // let mut last_time = SystemTime::now();
        let mut interval = time::interval(Duration::from_millis(GAME_LOOP_DELAY as u64));
        
        loop {
            
            interval.tick().await;
            let baits_c = Arc::clone(&self.baits);
            let player_c = Arc::clone(&self.players);
            // println!("Tick");
            let mut new_bait_arr = Vec::new();

            let mut cur_bait = baits_c.lock().await;
            let mut msg_new_bait_arr = String::new();
            let mut dead_players = Vec::new();

            if cur_bait.len() < MAX_BAITS as usize {
                let initial_bait = generate_bait(OFFSET_X + 10.0, TRUE_MAP_WIDTH - 10.0);
                let cl = initial_bait.clone();
                new_bait_arr.push(initial_bait);
                cur_bait.push(cl);
            }

            // Update all player positions
            let mut players_lock = player_c.lock().await;
            for player in players_lock.values_mut() {
                let move_x = player.move_x;
                let move_y = player.move_y;
                let window_w = player.window_w;
                let window_h = player.window_h;
                let plr_snake = player.get_snake();
                if plr_snake.accelerate && plr_snake.nodes.len() > SNAKE_INITIAL_LENGTH {
                    if plr_snake.accelerate_time < SNAKE_IT_IS_TIME_TO_SHORTER {
                        plr_snake.accelerate_time += 1;
                    } else {
                        plr_snake.accelerate_time = 0;

                        let last_node = &plr_snake.nodes[plr_snake.nodes.len() - 1];
                        let new_bait = Bait::new(
                            last_node.x,
                            last_node.y,
                            format!("{}", rand::random_range(0..MAX_BAIT_COLOR_RANGE)),
                            5.0,
                        );
                        let cl = new_bait.clone();
                        cur_bait.push(new_bait);
                        new_bait_arr.push(cl);

                        // Remove the last node (make snake shorter)
                        plr_snake.shorter();
                    }
                }

                plr_snake.move_snake(move_x, move_y, window_w as f64, window_h as f64);
            }

            for player in players_lock.values() {
                // If player is already dead, skip
                if dead_players.contains(&player.id) {
                    continue;
                }

                // Check against all other players
                for other_player in players_lock.values() {
                    if other_player == player {
                        continue; // A player cannot hit itself
                    }

                    let player_j_head = Rect {
                        top: other_player.snake.nodes[0].y - SNAKE_INITIAL_SIZE / 3.0,
                        left: other_player.snake.nodes[0].x - SNAKE_INITIAL_SIZE / 3.0,
                        right: other_player.snake.nodes[0].x + SNAKE_INITIAL_SIZE / 3.0,
                        bottom: other_player.snake.nodes[0].y + SNAKE_INITIAL_SIZE / 3.0,
                    };

                    // Check collision with each node of player i
                    let mut hit = false;
                    for k in 0..player.snake.nodes.len() {
                        let player_i_node = Rect {
                            top: player.snake.nodes[k].y - SNAKE_INITIAL_SIZE / 3.0,
                            left: player.snake.nodes[k].x - SNAKE_INITIAL_SIZE / 3.0,
                            right: player.snake.nodes[k].x + SNAKE_INITIAL_SIZE / 3.0,
                            bottom: player.snake.nodes[k].y + SNAKE_INITIAL_SIZE / 3.0,
                        };

                        if rect_intersect(&player_i_node, &player_j_head) {
                            hit = true;

                            // Generate baits from dead snake
                            let new_bait_on_dead = generate_mass_bait(&other_player.snake);
                            for bait in new_bait_on_dead {
                                let bt_c = bait.clone();
                                new_bait_arr.push(bait);
                                cur_bait.push(bt_c);
                            }

                            dead_players.push(other_player.id);

                            // Notify player about death
                            let death_msg = format!("{}8", COMM_START_NEW_MESS);
                            if let Err(e) = self
                                .socket
                                .send_to(death_msg.as_bytes(), other_player.addr)
                                .await
                            {
                                eprintln!("Failed to send welcome to {}: {}", other_player.addr, e);
                            }


                            break;
                        }
                    }

                    if hit {
                        break;
                    }
                }
            }

            // Inform all remaining players about dead players
            let mut msg_dead_players = String::new();
            for &dead_id in &dead_players {
                msg_dead_players.push_str(&format!("{}7,{}", COMM_START_NEW_MESS, dead_id));
            }

            // Send death notifications to all players
            if !msg_dead_players.is_empty() {
                for player in players_lock.values() {
                    if let Err(e) = self
                        .socket
                        .send_to(msg_dead_players.as_bytes(), player.addr)
                        .await
                    {
                        eprintln!("Failed to send welcome to {}: {}", player.addr, e);
                    }
                }
            }

            // Send new baits to all players
            if !msg_new_bait_arr.is_empty() {
                for player in players_lock.values() {
                    if let Err(e) = self
                        .socket
                        .send_to(msg_new_bait_arr.as_bytes(), player.addr)
                        .await
                    {
                        eprintln!("Failed to send welcome to {}: {}", player.addr, e);
                    }
                }
            }

            // Check if a player eats a bait
            let bt_lk = cur_bait.clone();
            let bait_key = &bt_lk.iter().enumerate();
            let mut deleted_baits = Vec::new();
            let mut msg_grown_players = String::new();

            for player in players_lock.values_mut() {
                let plr_id = player.id;
                let player_i_head = Rect {
                    top: player.snake.nodes[0].y - SNAKE_INITIAL_SIZE / 2.0,
                    left: player.snake.nodes[0].x - SNAKE_INITIAL_SIZE / 2.0,
                    right: player.snake.nodes[0].x + SNAKE_INITIAL_SIZE / 2.0,
                    bottom: player.snake.nodes[0].y + SNAKE_INITIAL_SIZE / 2.0,
                };
                let bk = bait_key.clone();
                for (idx, bait_tmp) in bk {
                    let bait_rect = Rect {
                        top: bait_tmp.y - bait_tmp.size / 2.0,
                        left: bait_tmp.x - bait_tmp.size / 2.0,
                        right: bait_tmp.x + bait_tmp.size / 2.0,
                        bottom: bait_tmp.y + bait_tmp.size / 2.0,
                    };

                    if rect_intersect(&player_i_head, &bait_rect) {
                        // Grow the snake
                        player.grow_player_snake();
                        msg_grown_players
                            .push_str(&format!("{}62,{}", COMM_START_NEW_MESS, plr_id));
                        cur_bait.remove(idx);
                        // bait::destroy(j);
                        deleted_baits.push(bait_tmp);
                    }
                }
            }
            // Inform players about deleted baits
            let mut msg_deleted_baits = String::new();
            for bait in &deleted_baits {
                msg_deleted_baits
                    .push_str(&format!("{}4,{},{}", COMM_START_NEW_MESS, bait.x, bait.y));
            }

            // Send bait deletion and growth notifications
            for player in players_lock.values_mut() {
                if !msg_deleted_baits.is_empty() {
                    if let Err(e) = self
                        .socket
                        .send_to(msg_deleted_baits.as_bytes(), player.addr)
                        .await
                    {
                        eprintln!("Failed to send welcome to {}: {}", player.addr, e);
                    }
                }

                if !msg_grown_players.is_empty() {
                    if let Err(e) = self
                        .socket
                        .send_to(msg_grown_players.as_bytes(), player.addr)
                        .await
                    {
                        eprintln!("Failed to send welcome to {}: {}", player.addr, e);
                    }
                }
            }

            for player in players_lock.values_mut() {
                let mut msg_update_player = format!("{}2,", COMM_START_NEW_MESS);

                for (j, node) in player.snake.nodes.iter().enumerate() {
                    msg_update_player.push_str(&format!("{:.4},{:.4}", node.x, node.y));
                    if j < player.snake.nodes.len() - 1 {
                        msg_update_player.push(',');
                    }
                }
                println!("Send msg_update_player {} to{}",msg_update_player,player.addr);
                if let Err(e) = self
                    .socket
                    .send_to(msg_update_player.as_bytes(), player.addr)
                    .await
                {
                    eprintln!("Failed to send welcome to {}: {}", player.addr, e);
                }
            }

            for player in players_lock.values() {
                let mut msg_update_enemies_position = String::new();

                for other_player in players_lock.values() {
                    if player == other_player {
                        continue;
                    }

                    msg_update_enemies_position
                        .push_str(&format!("{}6,{},", COMM_START_NEW_MESS, other_player.id));

                    for (k, node) in other_player.snake.nodes.iter().enumerate() {
                        msg_update_enemies_position
                            .push_str(&format!("{:.4},{:.4}", node.x, node.y));
                        if k < other_player.snake.nodes.len() - 1 {
                            msg_update_enemies_position.push(',');
                        }
                    }
                }

                if !msg_update_enemies_position.is_empty() {
                    if let Err(e) = self
                        .socket
                        .send_to(msg_update_enemies_position.as_bytes(), player.addr)
                        .await
                    {
                        eprintln!("Failed to send welcome to {}: {}", player.addr, e);
                    }
                }
            }

            for bait in &new_bait_arr {
                msg_new_bait_arr.push_str(&format!(
                    "{}3,{},{},{}",
                    COMM_START_NEW_MESS, bait.x, bait.y, bait.size
                ));
            }
            if !msg_new_bait_arr.is_empty() {
                for player in players_lock.values() {
                    if let Err(e) = self
                        .socket
                        .send_to(msg_new_bait_arr.as_bytes(), player.addr)
                        .await
                    {
                        eprintln!("Failed to send welcome to {}: {}", player.addr, e);
                    }
                }
            }

            // Clean up inactive players (UDP connection management)
            let inactive_players = self.clean_inactive_players(30, players_lock.clone()).await; // 30 seconds timeout
            for id in inactive_players {
                println!("Player {} disconnected due to inactivity", id);
                let msg = format!("{}7,{}", COMM_START_NEW_MESS, id);

                // Notify remaining players
                for player in players_lock.values() {
                    if player.addr != id {
                        if let Err(e) = self
                            .socket
                            .send_to(msg.clone().as_bytes(), player.addr)
                            .await
                        {
                            eprintln!("Failed to send welcome to {}: {}", player.addr, e);
                        }
                    }
                }
            }
        }
    }

    // Remove players that haven't been seen in a while (UDP connection management)
    pub async fn clean_inactive_players(&self, timeout_secs: u64, players_lock: HashMap<SocketAddr, Player>) -> Vec<SocketAddr> {
        let mut inactive_ids = Vec::new();
        let mut plr = players_lock;
        let players = plr.clone().into_iter();

        for (i, player) in players {
            let elapsed = player.last_seen.elapsed().as_secs();
            if elapsed > timeout_secs {
                inactive_ids.push(i);
            }
        }

        // Remove inactive players
        for id in &inactive_ids {
            // if *id < players.len() {
            //     players[*id] = None;
            // }
            plr.remove(id);
        }

        inactive_ids
    }
    /// Handle creation of a new player

    async fn create_player(&self, addr: SocketAddr, mut players_lock: MutexGuard<'_, HashMap<SocketAddr, Player>>) {
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

        for other_player in players_lock.values() {
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
        for other_player in players_lock.values() {
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

        println!("Total player(s): {}", players_lock.len());
        players_lock.insert(addr, new_player);
    }
}

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    // Bind UDP socket and create the game server
    let server = Arc::new(GameServer::new("0.0.0.0:5000").await?);

    // Start the listener task
    server.clone().start_listener();

    // Start the game loop in background
    let game_handle = server.clone().game_loop().await;
    println!("Running...");
    // Wait for Ctrl-C to shut down
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for ctrl-c");
    println!("Shutting down server...");

    // // Stop the game loop
    // game_handle.abort();

    Ok(())
}
