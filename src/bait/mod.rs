use crate::constants::*;

pub struct Bait {
    pub x: f64,
    pub y: f64,
    pub color: String,
    pub size: f64,
}


impl  Bait {
    pub fn new(x: f64, y: f64, color: String, size: f64) -> Bait {
        let new_bait = Bait {
            x,
            y,
            color,
            size,
        };
        new_bait
    }
}

impl Clone for Bait {
    fn clone(&self) -> Self {
        Bait {
            x: self.x,
            y: self.y,
            color: self.color.clone(),
            size: self.size,
        }
    }
} 