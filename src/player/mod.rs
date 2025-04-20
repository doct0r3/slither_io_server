pub mod snake;

use std::net::SocketAddr;
use std::time::Instant;

use snake::Snake;
// Import the real Snake type from our snake module

pub struct Player {
    pub id: u128,
    pub name: String,
    pub snake: Snake,
    pub addr: SocketAddr,
    pub move_x: f64,
    pub move_y: f64,
    pub window_w: u32,
    pub window_h: u32,
    pub last_seen: Instant,
    pub sent_pkt: u32,
    pub recv_pkt: u32,
}

impl Player {
    pub fn new(id: u128, name: String, snake: Snake, addr: SocketAddr) -> Player {
        Player {
            id,
            name,
            snake,
            addr,
            move_x: 0.0,
            move_y: 0.0,
            window_w: 0,
            window_h: 0,
            last_seen: Instant::now(),
            sent_pkt: 0,
            recv_pkt: 0,
        }
    }

    pub fn get_snake(&mut self) -> &mut Snake {
        &mut self.snake
    }

    pub fn update_xy(&mut self, x: f64, y: f64, win_w: u32, win_h: u32) {
        self.move_x = x;
        self.move_y = y;
        self.window_h = win_h;
        self.window_w = win_w;
    }

    pub fn update_last_seen(&mut self) {
        self.last_seen = std::time::Instant::now();
    }

    pub fn update_player_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn update_player_acceleration(&mut self, accelerate: bool) {
        self.snake.accelerate = accelerate;
    }

    pub fn update_player_pkt_stat(&mut self, send_pkt: u32, recv_pkt: u32) {
        self.sent_pkt = send_pkt;
        self.recv_pkt = recv_pkt;
    }

    pub fn grow_player_snake(&mut self) {
        self.snake.grow();
    }
}

// Implement Clone for Player
impl Clone for Player {
    fn clone(&self) -> Self {
        // Note: Cloning TcpStream isn't generally recommended
        // This is a simplified implementation for illustration
        Player {
            id: self.id,
            name: self.name.clone(),
            snake: self.snake.clone(),
            addr: self.addr,
            move_x: self.move_x,
            move_y: self.move_y,
            window_w: self.window_w,
            window_h: self.window_h,
            last_seen: self.last_seen,
            sent_pkt: self.sent_pkt,
            recv_pkt: self.recv_pkt,
        }
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.addr == other.addr
    }
}
