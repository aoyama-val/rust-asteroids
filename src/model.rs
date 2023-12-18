use rand::prelude::*;
use std::f32::consts::PI;
use std::time;

pub const SCREEN_WIDTH: usize = 640;
pub const SCREEN_HEIGHT: usize = 420;
pub const PLAYER_SIZE: u32 = 20;
pub const BULLET_SIZE: u32 = 5;

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
            x: (SCREEN_WIDTH / 2 - PLAYER_SIZE as usize / 2) as f32,
            y: (SCREEN_HEIGHT / 2 - PLAYER_SIZE as usize / 2) as f32,
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

        self.x = min_max_loop(0.0, self.x, SCREEN_WIDTH as f32);
        self.y = min_max_loop(0.0, self.y, SCREEN_WIDTH as f32);

        self.velocity *= 0.97;

        let rot_i32 = self.rot as i32;
        if rot_i32 == prev_rot_i32 {
            // いったん止まったように見えた後再度動くと不自然なので完全に0.0にする
            self.vrot = 0.0;
        } else {
            self.vrot *= 0.97;
        }
    }
}

#[derive(Clone)]
pub struct Asteroid {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub size: f32,
    pub should_remove: bool,
}

impl Asteroid {
    pub fn do_move(&mut self) {
        self.x += self.vx;
        self.y += self.vy;
        if self.x < 0.0 || self.x > SCREEN_WIDTH as f32 {
            self.should_remove = true;
        }
        if self.y < 0.0 || self.y > SCREEN_HEIGHT as f32 {
            self.should_remove = true;
        }
    }
}

#[derive(Clone)]
pub struct Bullet {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub should_remove: bool,
}

impl Bullet {
    pub fn do_move(&mut self) {
        self.x += self.vx;
        self.y += self.vy;
        if self.x < 0.0 || self.x > SCREEN_WIDTH as f32 {
            self.should_remove = true;
        }
        if self.y < 0.0 || self.y > SCREEN_HEIGHT as f32 {
            self.should_remove = true;
        }
    }
}

pub struct Game {
    pub rng: StdRng,
    pub is_over: bool,
    pub frame: i32,
    pub player: Player,
    pub score: i32,
    pub asteroids: Vec<Asteroid>,
    pub bullets: Vec<Bullet>,
    pub requested_sounds: Vec<&'static str>,
}

impl Game {
    pub fn new() -> Self {
        let now = time::SystemTime::now();
        let timestamp = now
            .duration_since(time::UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_secs();
        let rng = StdRng::seed_from_u64(timestamp);

        let game = Game {
            rng: rng,
            is_over: false,
            frame: 0,
            player: Player::new(),
            score: 0,
            asteroids: Vec::new(),
            bullets: Vec::new(),
            requested_sounds: Vec::new(),
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
            Command::Shoot => self.shoot(),
        }

        self.player.do_move();

        for bullet in &mut self.bullets {
            bullet.do_move();
        }

        for asteroid in &mut self.asteroids {
            asteroid.do_move();

            for bullet in &mut self.bullets {
                if is_collide(
                    bullet.x,
                    bullet.y,
                    BULLET_SIZE as f32,
                    BULLET_SIZE as f32,
                    asteroid.x,
                    asteroid.y,
                    asteroid.size,
                    asteroid.size,
                ) {
                    bullet.should_remove = true;
                    asteroid.should_remove = true;
                    self.score += 100;
                    // self.requested_sounds.push("hit.mp3");
                }
            }

            if is_collide(
                asteroid.x,
                asteroid.y,
                asteroid.size,
                asteroid.size,
                self.player.x,
                self.player.y,
                PLAYER_SIZE as f32,
                PLAYER_SIZE as f32,
            ) {
                self.is_over = true;
                self.requested_sounds.push("crash.wav");
            }
        }

        self.bullets = self
            .bullets
            .iter()
            .filter(|a| !a.should_remove)
            .map(|a| (*a).clone())
            .collect();

        self.asteroids = self
            .asteroids
            .iter()
            .filter(|a| !a.should_remove)
            .map(|a| (*a).clone())
            .collect();

        if self.rng.gen_bool(0.07) && self.asteroids.len() < 30 {
            self.spawn_asteroid();
        }

        self.frame += 1;
    }

    pub fn spawn_asteroid(&mut self) {
        let size = 8.0 + self.rng.gen::<f32>() * 15.0;
        let v = 2.5 * self.rng.gen::<f32>() * 2.0;
        let rot = (self.rng.gen::<f32>()) * 360.0;
        let vx = v * f32::cos(rot);
        let vy = v * f32::sin(rot) * -1.0;

        let x;
        let y;
        if self.rng.gen::<u32>() % 2 == 0 {
            if vx > 0.0 {
                x = 0.0;
            } else {
                x = SCREEN_WIDTH as f32 - 1.0;
            }
            y = self.rng.gen::<f32>() * SCREEN_HEIGHT as f32;
        } else {
            if vy > 0.0 {
                y = 0.0;
            } else {
                y = SCREEN_HEIGHT as f32 - 1.0;
            }
            x = self.rng.gen::<f32>() * SCREEN_WIDTH as f32;
        }

        let asteroid = Asteroid {
            x: x as f32,
            y: y as f32,
            size: size,
            vx: vx,
            vy: vy,
            should_remove: false,
        };
        self.asteroids.push(asteroid);
    }

    pub fn shoot(&mut self) {
        if self.bullets.len() >= 3 {
            return;
        }

        let v = 4.0;
        let rot = self.player.rot;
        let vx = v * f32::cos(deg2rad(rot));
        let vy = v * f32::sin(deg2rad(rot)) * -1.0;
        let bullet = Bullet {
            x: self.player.x + PLAYER_SIZE as f32 / 2.0,
            y: self.player.y + PLAYER_SIZE as f32 / 2.0,
            vx: vx,
            vy: vy,
            should_remove: false,
        };
        self.bullets.push(bullet);
        // self.requested_sounds.push("shoot.wav");
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

pub fn deg2rad(degree: f32) -> f32 {
    (2.0 * PI) * degree / 360.0
}

pub fn min_max_loop<T: PartialOrd + num_traits::NumOps>(min: T, value: T, max: T) -> T {
    if value < min {
        return value + max;
    }
    if value > max {
        return value - max;
    }
    value
}

pub fn is_collide(x1: f32, y1: f32, w1: f32, h1: f32, x2: f32, y2: f32, w2: f32, h2: f32) -> bool {
    return (x1 <= x2 + w2 && x2 <= x1 + w1) && (y1 <= y2 + h2 && y2 <= y1 + h1);
}
