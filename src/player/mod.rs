

pub mod snake;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::sync::Mutex;

// Import the real Snake type from our snake module

pub struct Player {
    pub id: String,
    pub name: String,
    pub score: i32,
    pub current_rank: i32,
    pub snake: Snake,
    pub addr: SocketAddr,
    pub move_x: f64,
    pub move_y: f64,
    pub window_w: u32,
    pub window_h: u32,
}

pub fn create(
    id: String,
    name: String,
    score: i32,
    current_rank: i32,
    snake: Snake,
    addr: SocketAddr,
) {
    let player = Player {
        id: id.clone(),
        name,
        score,
        current_rank,
        snake,
        addr,
        move_x: 0.0,
        move_y: 0.0,
        window_w: 0,
        window_h: 0,
    };
    PLAYERS.lock().unwrap().insert(id, player);
}

pub fn get_snake(id: &str) -> Option<Snake> {
    PLAYERS
        .lock()
        .unwrap()
        .get(id)
        .map(|player| player.snake.clone())
}

pub fn read(id: &str) -> Option<Player> {
    PLAYERS.lock().unwrap().get(id).cloned()
}

pub fn destroy(id: &str) {
    PLAYERS.lock().unwrap().remove(id);
}

pub fn keys() -> Vec<String> {
    PLAYERS.lock().unwrap().keys().cloned().collect()
}

pub fn length() -> usize {
    PLAYERS.lock().unwrap().len()
}

pub fn update_xy(id: &str, x: f64, y: f64) -> bool {
    if let Some(player) = PLAYERS.lock().unwrap().get_mut(id) {
        player.move_x = x;
        player.move_y = y;
        true
    } else {
        false
    }
}

pub fn find_id_by_socket(addr: &SocketAddr) -> Option<String> {
    let players = PLAYERS.lock().unwrap();

    for (id, player) in players.iter() {
        if player.ip == addr.ip().to_string() && player.port == addr.port() {
            return Some(id.clone());
        }
    }
    None
}

pub fn destroy_socket(player_id: &str) -> bool {
    if let Some(player) = PLAYERS.lock().unwrap().get_mut(player_id) {
        // In Rust, calling shutdown is more idiomatic than destroy
        let _ = player.socket.shutdown(std::net::Shutdown::Both);
        true
    } else {
        false
    }
}

// Implement Clone for Player
impl Clone for Player {
    fn clone(&self) -> Self {
        // Note: Cloning TcpStream isn't generally recommended
        // This is a simplified implementation for illustration
        Player {
            id: self.id.clone(),
            name: self.name.clone(),
            score: self.score,
            current_rank: self.current_rank,
            snake: self.snake.clone(),
            addr,
            move_x: self.move_x,
            move_y: self.move_y,
            window_w: self.window_w,
            window_h: self.window_h,
        }
    }
}

// Add this new function to update a player's data after modifying it
pub fn update(player: &Player) -> bool {
    let mut players = PLAYERS.lock().unwrap();
    if players.contains_key(&player.id) {
        players.insert(player.id.clone(), player.clone());
        true
    } else {
        false
    }
}
