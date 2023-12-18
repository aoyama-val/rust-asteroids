use rand::prelude::*;
use std::time;

pub const SCREEN_WIDTH: usize = 640;
pub const SCREEN_HEIGHT: usize = 420;

pub enum Command {
    None,
    Left,
    Right,
    Forward,
    Shoot,
}

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub rot: f32,
    pub vrot: f32,
}

impl Player {
    pub fn new() -> Self {
        let player = Player {
            x: (SCREEN_WIDTH / 2) as f32,
            y: (SCREEN_HEIGHT / 2) as f32,
            vx: 0.0,
            vy: 0.0,
            rot: 0.0,
            vrot: 0.0,
        };
        player
    }

    pub fn up(&mut self) {
        self.vy -= 1.8;
    }

    pub fn rotate(&mut self, degree: f32) {
        // self.rot = clamp_exclusive(0.0, self.rot + degree, 360.0);
        self.rot += degree;
        if self.rot < 0.0 {
            self.rot += 360.0;
        }
        if self.rot >= 360.0 {
            self.rot -= 360.0;
        }
    }
}

pub struct Game {
    pub rng: StdRng,
    pub is_over: bool,
    pub frame: i32,
    pub player: Player,
    pub score: i32,
}

impl Game {
    pub fn new() -> Self {
        let now = time::SystemTime::now();
        let timestamp = now
            .duration_since(time::UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_secs();
        let rng = StdRng::seed_from_u64(timestamp);

        let mut game = Game {
            rng: rng,
            is_over: false,
            frame: 0,
            player: Player::new(),
            score: 0,
        };

        game
    }

    pub fn update(&mut self, command: Command) {
        if self.is_over {
            return;
        }

        match command {
            Command::None => {}
            Command::Left => self.player.rotate(10.0),
            Command::Right => self.player.rotate(-10.0),
            Command::Forward => self.player.up(),
            Command::Shoot => {}
        }

        self.frame += 1;
    }
}

fn clamp<T: PartialOrd>(min: T, value: T, max: T) -> T {
    if value < min {
        return min;
    }
    if value > max {
        return max;
    }
    value
}

fn clamp_exclusive<T: PartialOrd>(min: T, value: T, max: T) -> T {
    if value < min {
        return min;
    }
    if value >= max {
        return max;
    }
    value
}
