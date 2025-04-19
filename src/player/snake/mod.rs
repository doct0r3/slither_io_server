use once_cell::sync::Lazy;
use rand::Rng;
use std::collections::HashMap;
use std::sync::Mutex;

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

impl Snake {
    pub fn new(length: i32, skin: String, speed: f64) -> Snake {
        let map_border_w = BORDER_WIDTH - MAP_WIDTH;
        let map_border_h = BORDER_HEIGHT - MAP_HEIGHT;
        let initial_x =
            rand::random_range(map_border_w / 2.0 + 500.0..map_border_w / 2.0 + MAP_WIDTH - 500.0);
        let initial_y =
            rand::random_range(map_border_h / 2.0 + 500.0..map_border_h / 2.0 + MAP_HEIGHT - 500.0);

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

    pub fn grow(&mut self) {
        if self.nodes.len() < 500 {
            let last_node = self.nodes.last().unwrap().clone();
            self.nodes.push(SnakeNode {
                x: last_node.x,
                y: last_node.y,
            });
        }
    }

    pub fn new_rotate_angle(&mut self, angle: f64) {
        self.rotate_angle = angle;
    }

    pub fn rotate(&mut self) {
        if self.rotate_angle > self.current_angle {
            self.current_angle =
                f64::min(self.rotate_angle, self.current_angle + SNAKE_ROTATE_SPEED);
        } else {
            self.current_angle =
                f64::max(self.rotate_angle, self.current_angle - SNAKE_ROTATE_SPEED);
        }
    }
    pub fn move_snake(&mut self, to_x: f64, to_y: f64, center_x: f64, center_y: f64) {
        if SERVER_CURRENT_UPDATE_PLAYER_METHOD == 1 {
            let n = self.nodes.len();

            for i in (1..n).rev() {
                self.nodes[i].x = self.nodes[i - 1].x;
                self.nodes[i].y = self.nodes[i - 1].y;
            }

            let dx = to_x - center_x / 2.0;
            let dy = to_y - center_y / 2.0;

            let dist = (dx * dx + dy * dy).sqrt();

            let norm_x = dx / if dist == 0.0 { 1.0 } else { dist };
            let norm_y = dy / if dist == 0.0 { 1.0 } else { dist };

            let vel_x = norm_x * SNAKE_SPEED;
            let vel_y = norm_y * SNAKE_SPEED;

            self.nodes[0].x += vel_x;
            self.nodes[0].y += vel_y;

            // Limit by MAP_BORDER
            if self.nodes[0].x - SNAKE_INITIAL_SIZE / 2.0 < OFFSET_X {
                self.nodes[0].x = OFFSET_X + SNAKE_INITIAL_SIZE / 2.0;
            }
            if self.nodes[0].y - SNAKE_INITIAL_SIZE / 2.0 < OFFSET_Y {
                self.nodes[0].y = OFFSET_Y + SNAKE_INITIAL_SIZE / 2.0;
            }
            if self.nodes[0].x + SNAKE_INITIAL_SIZE / 2.0 > TRUE_MAP_WIDTH {
                self.nodes[0].x = TRUE_MAP_WIDTH - SNAKE_INITIAL_SIZE / 2.0;
            }
            if self.nodes[0].y + SNAKE_INITIAL_SIZE / 2.0 > TRUE_MAP_HEIGHT {
                self.nodes[0].y = TRUE_MAP_HEIGHT - SNAKE_INITIAL_SIZE / 2.0;
            }
        } else if SERVER_CURRENT_UPDATE_PLAYER_METHOD == 2 {
            // new method
            let n = self.nodes.len();

            for i in (1..n).rev() {
                let dx = self.nodes[i - 1].x - self.nodes[i].x;
                let dy = self.nodes[i - 1].y - self.nodes[i].y;
                let dist = (dx * dx + dy * dy).sqrt();
                let node_dist = dist / SNAKE_NODE_INITIAL_DISTANCE;
                let speed = (if self.accelerate {
                    SNAKE_SPEED_ACCELERATE * SNAKE_SPEED
                } else {
                    SNAKE_SPEED
                }) * node_dist;

                // Normalize direction
                let norm_x = dx / if dist == 0.0 { 0.1 } else { dist };
                let norm_y = dy / if dist == 0.0 { 0.1 } else { dist };

                // Calculate velocity
                let vel_x = norm_x * speed;
                let vel_y = norm_y * speed;

                // Update position
                self.nodes[i].x += vel_x;
                self.nodes[i].y += vel_y;

                // Apply map bounds
                if self.nodes[i].x - SNAKE_INITIAL_SIZE / 2.0 < OFFSET_X {
                    self.nodes[i].x = OFFSET_X + SNAKE_INITIAL_SIZE / 2.0;
                }
                if self.nodes[i].y - SNAKE_INITIAL_SIZE / 2.0 < OFFSET_Y {
                    self.nodes[i].y = OFFSET_Y + SNAKE_INITIAL_SIZE / 2.0;
                }
                if self.nodes[i].x + SNAKE_INITIAL_SIZE / 2.0 > TRUE_MAP_WIDTH {
                    self.nodes[i].x = TRUE_MAP_WIDTH - SNAKE_INITIAL_SIZE / 2.0;
                }
                if self.nodes[i].y + SNAKE_INITIAL_SIZE / 2.0 > TRUE_MAP_HEIGHT {
                    self.nodes[i].y = TRUE_MAP_HEIGHT - SNAKE_INITIAL_SIZE / 2.0;
                }
            }

            // Move head
            let dx = to_x - center_x / 2.0;
            let dy = to_y - center_y / 2.0;
            let dist = (dx * dx + dy * dy).sqrt();
            let norm_x = dx / if dist == 0.0 { 1.0 } else { dist };
            let norm_y = dy / if dist == 0.0 { 1.0 } else { dist };

            let vel_x = norm_x
                * if self.accelerate {
                    SNAKE_SPEED_ACCELERATE * SNAKE_SPEED
                } else {
                    SNAKE_SPEED
                };
            let vel_y = norm_y
                * if self.accelerate {
                    SNAKE_SPEED_ACCELERATE * SNAKE_SPEED
                } else {
                    SNAKE_SPEED
                };

            self.nodes[0].x += vel_x;
            self.nodes[0].y += vel_y;

            // Apply map bounds to head
            if self.nodes[0].x - SNAKE_INITIAL_SIZE / 2.0 < OFFSET_X {
                self.nodes[0].x = OFFSET_X + SNAKE_INITIAL_SIZE / 2.0;
            }
            if self.nodes[0].y - SNAKE_INITIAL_SIZE / 2.0 < OFFSET_Y {
                self.nodes[0].y = OFFSET_Y + SNAKE_INITIAL_SIZE / 2.0;
            }
            if self.nodes[0].x + SNAKE_INITIAL_SIZE / 2.0 > TRUE_MAP_WIDTH {
                self.nodes[0].x = TRUE_MAP_WIDTH - SNAKE_INITIAL_SIZE / 2.0;
            }
            if self.nodes[0].y + SNAKE_INITIAL_SIZE / 2.0 > TRUE_MAP_HEIGHT {
                self.nodes[0].y = TRUE_MAP_HEIGHT - SNAKE_INITIAL_SIZE / 2.0;
            }
        }
    }


    pub fn shorter(&mut self) {
        if !self.nodes.is_empty() {
            self.nodes.pop();
        }
    }
}



fn random(low: f64, high: f64) -> f64 {
    rand::random_range(low..high)
}
