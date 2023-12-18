use rand::prelude::*;
use std::f32::consts::PI;
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
    pub velocity: f32,
    pub rot: f32, // 角度。数学と同じく、右向きが0で反時計回り。[0 - 360)度
    pub vrot: f32,
}

impl Player {
    pub fn new() -> Self {
        let player = Player {
            x: (SCREEN_WIDTH / 2) as f32,
            y: (SCREEN_HEIGHT / 2) as f32,
            velocity: 0.0,
            rot: 90.0,
            vrot: 0.0,
        };
        player
    }

    pub fn up(&mut self) {
        self.velocity = 1.0;
    }

    pub fn rotate(&mut self, degree: f32) {
        self.vrot = degree;
    }

    pub fn do_move(&mut self) {
        let prev_rot_i32 = self.rot as i32;
        self.rot += self.vrot;
        if self.rot < 0.0 {
            self.rot += 360.0;
        }
        if self.rot >= 360.0 {
            self.rot -= 360.0;
        }

        self.x += self.velocity * f32::cos(deg2rad(self.rot));
        self.y += self.velocity * f32::sin(deg2rad(self.rot)) * -1.0;
        self.velocity *= 0.97;

        let rot_i32 = self.rot as i32;
        if rot_i32 == prev_rot_i32 {
            self.vrot = 0.0;
        } else {
            self.vrot *= 0.97;
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
            Command::Left => self.player.rotate(5.0),
            Command::Right => self.player.rotate(-5.0),
            Command::Forward => self.player.up(),
            Command::Shoot => {}
        }

        self.player.do_move();

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

pub fn deg2rad(degree: f32) -> f32 {
    (2.0 * PI) * degree / 360.0
}
