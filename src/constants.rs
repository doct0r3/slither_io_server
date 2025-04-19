// SNAKE
pub const SNAKE_INITIAL_LENGTH: usize = 5;
pub const SNAKE_SPEED: f64 = 2.0;
pub const SNAKE_SPEED_ACCELERATE: f64 = 4.0;
pub const SNAKE_SKIN_COLOR_RANGE: i32 = 255;
pub const SNAKE_ROTATE_SPEED: f64 = 5.0;
pub const SNAKE_NODE_SPACE: f64 = 0.0;
pub const SNAKE_NODE_INITIAL_DISTANCE: f64 = 7.071; // sqrt(50)
pub const SNAKE_INITIAL_SIZE: f64 = 17.0;
pub const SNAKE_IT_IS_TIME_TO_SHORTER: i32 = 20;

// BAIT
pub const MAX_BAIT_COLOR_RANGE: i32 = 255;
pub const MAX_BAIT_SIZE: i32 = 10;
pub const MIN_BAITS: i32 = 0;
pub const MAX_BAITS: i32 = 1000;
pub const MAX_BAITS_SIZE_ON_DEAD: i32 = 15;

// MAP
pub const MAP_WIDTH: f64 = 2000.0;
pub const MAP_HEIGHT: f64 = 2000.0;
pub const BORDER_WIDTH: f64 = 4000.0;
pub const BORDER_HEIGHT: f64 = 4000.0;
pub const OFFSET_X: f64 = 800.0;
pub const OFFSET_Y: f64 = 800.0;
pub const TRUE_MAP_WIDTH: f64 = 3200.0;
pub const TRUE_MAP_HEIGHT: f64 = 3200.0;

// GAME
pub const GAME_LOOP_DELAY: i32 = 50;
pub const SERVER_IP: &str = "0.0.0.0";
pub const SERVER_PORT: u16 = 5000;
pub const SERVER_CURRENT_UPDATE_PLAYER_METHOD: i32 = 2;
pub const SERVER_CURRENT_SENDING_PLAYER_METHOD: i32 = 2;
pub const SERVER_UPDATE_ENEMY_METHOD: i32 = 6;

// COMMAND
pub const COMM_START_NEW_MESS: &str = "$";
pub const COMM_NEW_SNAKE: &str = "1,";
pub const COMM_UPDATE_SNAKE: &str = "2,";
pub const COMM_UPDATE_SNAKE_HEAD_ONLY: &str = "21,";
pub const COMM_NEW_BAIT: &str = "3,";
pub const COMM_DELETE_BAIT: &str = "4,";
pub const COMM_NEW_ENEMY: &str = "5,";
pub const COMM_UPDATE_ENEMY: &str = "6,";
pub const COMM_DEAD_ENEMY: &str = "7,";
pub const COMM_DIE: &str = "8,";
pub const COMM_ENEMY_NAME: &str = "9,";
pub const COMM_SNAKE_ACCELERATING: &str = "10,"; 