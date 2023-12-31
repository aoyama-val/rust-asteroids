use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mixer;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, SystemTime};
mod model;
use crate::model::*;

const FPS: u32 = 30;

struct Image<'a> {
    texture: Texture<'a>,
    w: u32,
    h: u32,
}

impl<'a> Image<'a> {
    fn new(texture: Texture<'a>) -> Self {
        let q = texture.query();
        let image = Image {
            texture,
            w: q.width,
            h: q.height,
        };
        image
    }
}

struct Resources<'a> {
    images: HashMap<String, Image<'a>>,
    chunks: HashMap<String, sdl2::mixer::Chunk>,
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;

    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem
        .window("rust-asteroids", SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    sdl_context.mouse().show_cursor(false);

    init_mixer();

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_blend_mode(BlendMode::Blend);

    let texture_creator = canvas.texture_creator();
    let mut resources = load_resources(&texture_creator, &mut canvas);

    let mut event_pump = sdl_context.event_pump()?;

    let mut game = Game::new();

    println!("Keys:");
    println!("    Up    : Move player forward");
    println!("    Left  : Rotate player left");
    println!("    Right : Rotate player right");
    println!("    Shift : Shoot");
    println!("    Space : Restart when game over");

    'running: loop {
        let started = SystemTime::now();

        let mut command = Command::None;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    if game.is_over {
                        game = Game::new();
                    }
                }
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => {
                    command = match code {
                        Keycode::Left => Command::Left,
                        Keycode::Right => Command::Right,
                        Keycode::Up => Command::Forward,
                        Keycode::LShift => Command::Shoot,
                        Keycode::RShift => Command::Shoot,
                        _ => Command::None,
                    };
                }
                _ => {}
            }
        }
        game.update(command);
        render(&mut canvas, &game, &mut resources)?;

        play_sounds(&mut game, &resources);

        let finished = SystemTime::now();
        let elapsed = finished.duration_since(started).unwrap();
        let frame_duration = Duration::new(0, 1_000_000_000u32 / FPS);
        if elapsed < frame_duration {
            ::std::thread::sleep(frame_duration - elapsed)
        }
    }

    Ok(())
}

fn init_mixer() {
    let chunk_size = 1_024;
    mixer::open_audio(
        mixer::DEFAULT_FREQUENCY,
        mixer::DEFAULT_FORMAT,
        mixer::DEFAULT_CHANNELS,
        chunk_size,
    )
    .expect("cannot open audio");
    let _mixer_context = mixer::init(mixer::InitFlag::MP3).expect("cannot init mixer");
}

fn load_resources<'a>(
    texture_creator: &'a TextureCreator<WindowContext>,
    canvas: &mut Canvas<Window>,
) -> Resources<'a> {
    let mut resources = Resources {
        images: HashMap::new(),
        chunks: HashMap::new(),
    };

    // create player texture
    let mut player_texture = texture_creator
        .create_texture(
            None,
            sdl2::render::TextureAccess::Target,
            PLAYER_SIZE,
            PLAYER_SIZE,
        )
        .unwrap();
    canvas
        .with_texture_canvas(&mut player_texture, |texture_canvas| {
            texture_canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
            texture_canvas
                .draw_line(Point::new(9, 0), Point::new(2, 19))
                .unwrap();
            texture_canvas
                .draw_line(Point::new(2, 19), Point::new(16, 19))
                .unwrap();
            texture_canvas
                .draw_line(Point::new(16, 19), Point::new(9, 0))
                .unwrap();
        })
        .unwrap();
    let player_image = Image::new(player_texture);
    resources.images.insert("player".to_string(), player_image);

    // create asteroid texture
    let asteroid_texture_size = 20;
    let mut asteroid_texture = texture_creator
        .create_texture(
            None,
            sdl2::render::TextureAccess::Target,
            asteroid_texture_size,
            asteroid_texture_size,
        )
        .unwrap();
    canvas
        .with_texture_canvas(&mut asteroid_texture, |texture_canvas| {
            texture_canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
            texture_canvas
                .draw_rect(Rect::new(
                    0,
                    0,
                    asteroid_texture_size,
                    asteroid_texture_size,
                ))
                .unwrap();
        })
        .unwrap();
    let asteroid_image = Image::new(asteroid_texture);
    resources
        .images
        .insert("asteroid".to_string(), asteroid_image);

    let image_paths = ["numbers.bmp"];
    for path in image_paths {
        let full_path = "resources/image/".to_string() + path;
        let temp_surface = sdl2::surface::Surface::load_bmp(Path::new(&full_path)).unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&temp_surface)
            .expect(&format!("cannot load image: {}", path));

        let image = Image::new(texture);
        resources.images.insert(path.to_string(), image);
    }

    let sound_paths = ["crash.wav"];
    for path in sound_paths {
        let full_path = "resources/sound/".to_string() + path;
        let chunk =
            mixer::Chunk::from_file(full_path).expect(&format!("cannot load sound: {}", path));
        resources.chunks.insert(path.to_string(), chunk);
    }

    resources
}

fn render(
    canvas: &mut Canvas<Window>,
    game: &Game,
    resources: &mut Resources,
) -> Result<(), String> {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    // render player
    let player_image = resources.images.get_mut("player").unwrap();
    canvas
        .copy_ex(
            &player_image.texture,
            None,
            Rect::new(
                game.player.x as i32,
                game.player.y as i32,
                PLAYER_SIZE,
                PLAYER_SIZE,
            ),
            (game.player.rot + 90.0) as f64, /* SDLのangleは時計回りが正 */
            Point::new(PLAYER_SIZE as i32 / 2, PLAYER_SIZE as i32 / 2),
            false,
            false,
        )
        .unwrap();

    // render bullets
    canvas.set_draw_color(Color::RGB(255, 255, 0));
    for bullet in &game.bullets {
        canvas.draw_rect(Rect::new(
            bullet.x as i32,
            bullet.y as i32,
            BULLET_SIZE,
            BULLET_SIZE,
        ))?;
    }

    // render asteroids
    let asteroid_image = resources.images.get_mut("asteroid").unwrap();
    for asteroid in &game.asteroids {
        canvas
            .copy_ex(
                &asteroid_image.texture,
                None,
                Rect::new(
                    asteroid.x as i32,
                    asteroid.y as i32,
                    asteroid.size as u32,
                    asteroid.size as u32,
                ),
                (asteroid.rot + 90.0) as f64, /* SDLのangleは時計回りが正 */
                Point::new(asteroid.size as i32 / 2, asteroid.size as i32 / 2),
                false,
                false,
            )
            .unwrap();
    }

    if game.is_over {
        canvas.set_draw_color(Color::RGBA(255, 0, 0, 128));
        canvas.fill_rect(Rect::new(0, 0, SCREEN_WIDTH as u32, SCREEN_HEIGHT as u32))?;
    }

    render_number(
        canvas,
        resources,
        SCREEN_WIDTH as i32 - 8 * 8,
        0,
        format!("{0: >8}", game.score),
    );

    canvas.present();

    Ok(())
}

fn render_number(
    canvas: &mut Canvas<Window>,
    resources: &Resources,
    x: i32,
    y: i32,
    numstr: String,
) {
    let mut x = x;
    let image = resources.images.get("numbers.bmp").unwrap();
    let digit_width_in_px = 8;
    for c in numstr.chars() {
        if 0x30 <= c as i32 && c as i32 <= 0x39 {
            canvas
                .copy(
                    &image.texture,
                    Rect::new(
                        digit_width_in_px * (c as i32 - 0x30),
                        0,
                        digit_width_in_px as u32,
                        image.h,
                    ),
                    Rect::new(x, y, digit_width_in_px as u32, image.h),
                )
                .unwrap();
        }
        x += digit_width_in_px;
    }
}

fn play_sounds(game: &mut Game, resources: &Resources) {
    for sound_key in &game.requested_sounds {
        let chunk = resources
            .chunks
            .get(&sound_key.to_string())
            .expect("cannot get sound");
        sdl2::mixer::Channel::all()
            .play(&chunk, 0)
            .expect("cannot play sound");
    }
    game.requested_sounds = Vec::new();
}
