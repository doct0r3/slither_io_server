use std::sync::Mutex;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use rand::Rng;

// Import constants from the dedicated module
use crate::constants::*;



#[derive(Clone)]
pub struct SnakeNode {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone)]
pub struct Snake {
    pub length: i32,
    pub skin: String,
    pub speed: f64,
    pub current_speed_sec: f64,
    pub nodes: Vec<SnakeNode>,
    pub current_angle: f64,
    pub rotate_angle: f64,
    pub is_dead: bool,
    pub accelerate: bool,
    pub accelerate_time: i32,
}

static SNAKES: Lazy<Mutex<HashMap<String, Snake>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub fn create(length: i32, skin: String, speed: f64) -> Snake {
    let map_border_w = BORDER_WIDTH - MAP_WIDTH;
    let map_border_h = BORDER_HEIGHT - MAP_HEIGHT;
    
    let mut rng = rand::thread_rng();
    let initial_x = rng.gen_range(map_border_w / 2.0 + 500.0..map_border_w / 2.0 + MAP_WIDTH - 500.0);
    let initial_y = rng.gen_range(map_border_h / 2.0 + 500.0..map_border_h / 2.0 + MAP_HEIGHT - 500.0);
    
    let default_nodes = create_first_five_nodes(initial_x, initial_y);
    
    Snake {
        length,
        skin,
        speed,
        current_speed_sec: 0.0,
        nodes: default_nodes,
        current_angle: 0.0,
        rotate_angle: 0.0,
        is_dead: false,
        accelerate: false,
        accelerate_time: 0,
    }
}

pub fn read(id: &str) -> Option<Snake> {
    SNAKES.lock().unwrap().get(id).cloned()
}

pub fn destroy(id: &str) {
    SNAKES.lock().unwrap().remove(id);
}

pub fn keys() -> Vec<String> {
    SNAKES.lock().unwrap().keys().cloned().collect()
}

pub fn length() -> usize {
    SNAKES.lock().unwrap().len()
}

fn create_first_five_nodes(initial_x: f64, initial_y: f64) -> Vec<SnakeNode> {
    let mut nodes = Vec::new();
    
    nodes.push(SnakeNode {
        x: initial_x,
        y: initial_y,
    });
    
    for _ in 1..SNAKE_INITIAL_LENGTH {
        let last_node = nodes.last().unwrap();
        nodes.push(SnakeNode {
            x: last_node.x + SNAKE_NODE_SPACE,
            y: last_node.y + SNAKE_NODE_SPACE,
        });
    }
    
    nodes
}

pub fn grow(snake: &mut Snake) {
    if snake.nodes.len() < 500 {
        let last_node = snake.nodes.last().unwrap().clone();
        snake.nodes.push(SnakeNode {
            x: last_node.x,
            y: last_node.y,
        });
    }
}

pub fn new_rotate_angle(snake: &mut Snake, angle: f64) {
    snake.rotate_angle = angle;
}

pub fn rotate(snake: &mut Snake) {
    if snake.rotate_angle > snake.current_angle {
        snake.current_angle = f64::min(snake.rotate_angle, snake.current_angle + SNAKE_ROTATE_SPEED);
    } else {
        snake.current_angle = f64::max(snake.rotate_angle, snake.current_angle - SNAKE_ROTATE_SPEED);
    }
}

pub fn move_snake(snake: &mut Snake, to_x: f64, to_y: f64, center_x: f64, center_y: f64) {
    if SERVER_CURRENT_UPDATE_PLAYER_METHOD == 1 {
        let n = snake.nodes.len();
        
        for i in (1..n).rev() {
            snake.nodes[i].x = snake.nodes[i - 1].x;
            snake.nodes[i].y = snake.nodes[i - 1].y;
        }
        
        let dx = to_x - center_x / 2.0;
        let dy = to_y - center_y / 2.0;
        
        let dist = (dx * dx + dy * dy).sqrt();
        
        let norm_x = dx / if dist == 0.0 { 1.0 } else { dist };
        let norm_y = dy / if dist == 0.0 { 1.0 } else { dist };
        
        let vel_x = norm_x * SNAKE_SPEED;
        let vel_y = norm_y * SNAKE_SPEED;
        
        snake.nodes[0].x += vel_x;
        snake.nodes[0].y += vel_y;
        
        // Limit by MAP_BORDER
        if snake.nodes[0].x - SNAKE_INITIAL_SIZE / 2.0 < OFFSET_X {
            snake.nodes[0].x = OFFSET_X + SNAKE_INITIAL_SIZE / 2.0;
        }
        if snake.nodes[0].y - SNAKE_INITIAL_SIZE / 2.0 < OFFSET_Y {
            snake.nodes[0].y = OFFSET_Y + SNAKE_INITIAL_SIZE / 2.0;
        }
        if snake.nodes[0].x + SNAKE_INITIAL_SIZE / 2.0 > TRUE_MAP_WIDTH {
            snake.nodes[0].x = TRUE_MAP_WIDTH - SNAKE_INITIAL_SIZE / 2.0;
        }
        if snake.nodes[0].y + SNAKE_INITIAL_SIZE / 2.0 > TRUE_MAP_HEIGHT {
            snake.nodes[0].y = TRUE_MAP_HEIGHT - SNAKE_INITIAL_SIZE / 2.0;
        }
    } else if SERVER_CURRENT_UPDATE_PLAYER_METHOD == 2 {
        // new method
        let n = snake.nodes.len();
        
        for i in (1..n).rev() {
            let dx = snake.nodes[i - 1].x - snake.nodes[i].x;
            let dy = snake.nodes[i - 1].y - snake.nodes[i].y;
            let dist = (dx * dx + dy * dy).sqrt();
            let node_dist = dist / SNAKE_NODE_INITIAL_DISTANCE;
            let speed = (if snake.accelerate { SNAKE_SPEED_ACCELERATE * SNAKE_SPEED } else { SNAKE_SPEED }) * node_dist;
            
            // Normalize direction
            let norm_x = dx / if dist == 0.0 { 0.1 } else { dist };
            let norm_y = dy / if dist == 0.0 { 0.1 } else { dist };
            
            // Calculate velocity
            let vel_x = norm_x * speed;
            let vel_y = norm_y * speed;
            
            // Update position
            snake.nodes[i].x += vel_x;
            snake.nodes[i].y += vel_y;
            
            // Apply map bounds
            if snake.nodes[i].x - SNAKE_INITIAL_SIZE / 2.0 < OFFSET_X {
                snake.nodes[i].x = OFFSET_X + SNAKE_INITIAL_SIZE / 2.0;
            }
            if snake.nodes[i].y - SNAKE_INITIAL_SIZE / 2.0 < OFFSET_Y {
                snake.nodes[i].y = OFFSET_Y + SNAKE_INITIAL_SIZE / 2.0;
            }
            if snake.nodes[i].x + SNAKE_INITIAL_SIZE / 2.0 > TRUE_MAP_WIDTH {
                snake.nodes[i].x = TRUE_MAP_WIDTH - SNAKE_INITIAL_SIZE / 2.0;
            }
            if snake.nodes[i].y + SNAKE_INITIAL_SIZE / 2.0 > TRUE_MAP_HEIGHT {
                snake.nodes[i].y = TRUE_MAP_HEIGHT - SNAKE_INITIAL_SIZE / 2.0;
            }
        }
        
        // Move head
        let dx = to_x - center_x / 2.0;
        let dy = to_y - center_y / 2.0;
        let dist = (dx * dx + dy * dy).sqrt();
        let norm_x = dx / if dist == 0.0 { 1.0 } else { dist };
        let norm_y = dy / if dist == 0.0 { 1.0 } else { dist };
        
        let vel_x = norm_x * if snake.accelerate { SNAKE_SPEED_ACCELERATE * SNAKE_SPEED } else { SNAKE_SPEED };
        let vel_y = norm_y * if snake.accelerate { SNAKE_SPEED_ACCELERATE * SNAKE_SPEED } else { SNAKE_SPEED };
        
        snake.nodes[0].x += vel_x;
        snake.nodes[0].y += vel_y;
        
        // Apply map bounds to head
        if snake.nodes[0].x - SNAKE_INITIAL_SIZE / 2.0 < OFFSET_X {
            snake.nodes[0].x = OFFSET_X + SNAKE_INITIAL_SIZE / 2.0;
        }
        if snake.nodes[0].y - SNAKE_INITIAL_SIZE / 2.0 < OFFSET_Y {
            snake.nodes[0].y = OFFSET_Y + SNAKE_INITIAL_SIZE / 2.0;
        }
        if snake.nodes[0].x + SNAKE_INITIAL_SIZE / 2.0 > TRUE_MAP_WIDTH {
            snake.nodes[0].x = TRUE_MAP_WIDTH - SNAKE_INITIAL_SIZE / 2.0;
        }
        if snake.nodes[0].y + SNAKE_INITIAL_SIZE / 2.0 > TRUE_MAP_HEIGHT {
            snake.nodes[0].y = TRUE_MAP_HEIGHT - SNAKE_INITIAL_SIZE / 2.0;
        }
    }
}

pub fn shorter(snake: &mut Snake) {
    if !snake.nodes.is_empty() {
        snake.nodes.pop();
    }
}

fn random(low: f64, high: f64) -> f64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(low..high)
} 
