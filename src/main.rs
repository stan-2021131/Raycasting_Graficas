mod caster;
mod framebuffer;
mod maze;
mod player;
mod line;
mod texture;
mod sprite;
mod render;
mod sound;

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::time::{Duration, Instant};

use crate::framebuffer::Framebuffer;
use crate::maze::{is_goal, load_maze};
use crate::player::process_events;
use crate::render::{render_2d, render_3d, render_image, render_minimap, render_sprite, BLOCK_SIZE};
use crate::sound::AudioPlayer;
use crate::texture::{get_texture, load_textures, Texture};

enum GameState {
    Menu(usize),
    Controls,
    Playing,
    Completed,
}

fn main() {
    let window_width = 1000; // ancho de la ventana
    let window_height = 700; // alto de la ventana
    let framebuffer_width = 1300; // ancho del framebuffer
    let framebuffer_height = 900; // alto del framebuffer
    let frame_delay = Duration::from_micros(66667); // target de 66.67ms para 15fps
    let mut mode_3d = true; // modo 2d o 3d
    let mut last_mouse_x: Option<f32> = None; // ultima posicion X del mouse
    let mut current_level_path = String::from("./levels/maze.txt");

    // carga del laberinto una vez al inicio.
    let (mut maze, mut player, mut goal) = load_maze(&current_level_path, BLOCK_SIZE);

    // carga de texturas una sola vez.
    let textures = load_textures();
    let goal_texture = get_texture(&textures, 'g');

    // carga del audio al iniciar el programa.
    let mut audio_player = AudioPlayer::new();

    let victory_texture = Texture::new("./textures/screens/success.png");
    let menu_textures = vec![
        Texture::new("./textures/screens/level_1.png"),
        Texture::new("./textures/screens/level_2.png"),
        Texture::new("./textures/screens/level_3.png"),
        Texture::new("./textures/screens/controls.png"),
        Texture::new("./textures/screens/exit.png"),
    ];
    let controls_texture = Texture::new("./textures/screens/controls_1.png");

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    framebuffer.set_background_color(0x000000);

    let mut window = Window::new(
        "Maze Runner",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    let mut game_state = GameState::Menu(0);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let frame_start = Instant::now();

        match game_state {
            GameState::Menu(ref mut selected) => {
                // La musica solo debe sonar dentro del laberinto.
                audio_player.stop_music();

                if window.is_key_pressed(Key::W, KeyRepeat::No) {
                    *selected = selected.saturating_sub(1);
                }
                if window.is_key_pressed(Key::S, KeyRepeat::No) {
                    if *selected < 4 {
                        *selected += 1;
                    }
                }

                render_image(&mut framebuffer, &menu_textures[*selected]);

                if window.is_key_pressed(Key::Enter, KeyRepeat::No) {
                    match *selected {
                        0 | 1 | 2 => {
                            current_level_path = match *selected {
                                0 => String::from("./levels/maze.txt"),
                                1 => String::from("./levels/maze_2.txt"),
                                _ => String::from("./levels/maze_3.txt"),
                            };

                            let (new_maze, new_player, new_goal) =
                                load_maze(&current_level_path, BLOCK_SIZE);
                            maze = new_maze;
                            player = new_player;
                            goal = new_goal;

                            audio_player.start_music();
                            game_state = GameState::Playing;
                        }
                        3 => {
                            game_state = GameState::Controls;
                        }
                        4 => {
                            break;
                        }
                        _ => {}
                    }
                }

                window
                    .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
                    .unwrap();
            }
            GameState::Controls => {
                audio_player.stop_music();
                render_image(&mut framebuffer, &controls_texture);

                if window.is_key_pressed(Key::Enter, KeyRepeat::No) {
                    game_state = GameState::Menu(3);
                }

                window
                    .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
                    .unwrap();
            }
            GameState::Playing => {
                // Si ya entramos una vez al laberinto, solo reanudamos la musica; no la recargamos.
                audio_player.start_music();

                if window.is_key_pressed(Key::M, KeyRepeat::No) {
                    mode_3d = !mode_3d;
                }

                // Comparamos la posicion antes y despues del input para saber si hubo desplazamiento real.
                let previous_position = player.pos;
                process_events(&mut window, &mut player, &maze, BLOCK_SIZE, &mut last_mouse_x);

                if player.pos != previous_position {
                    audio_player.try_play_footstep();
                }

                if is_goal(&maze, player.pos.x, player.pos.y, BLOCK_SIZE) {
                    audio_player.stop_music();
                    game_state = GameState::Completed;
                } else {
                    framebuffer.clear();

                    if mode_3d {
                        let z_buffer = render_3d(&mut framebuffer, &maze, &player, &textures);
                        render_sprite(&mut framebuffer, &player, &goal, &goal_texture, &z_buffer);
                        render_minimap(&mut framebuffer, &maze, &player);
                    } else {
                        render_2d(&mut framebuffer, &maze, &player);
                    }

                    window
                        .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
                        .unwrap();
                }
            }
            GameState::Completed => {
                audio_player.stop_music();
                render_image(&mut framebuffer, &victory_texture);

                if window.is_key_pressed(Key::Enter, KeyRepeat::No) {
                    game_state = GameState::Menu(0);
                }

                window
                    .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
                    .unwrap();
            }
        }

        let elapsed = frame_start.elapsed();

        if elapsed < frame_delay {
            std::thread::sleep(frame_delay - elapsed);
        }

        let total_frame_time = frame_start.elapsed();
        let fps = 1.0 / total_frame_time.as_secs_f32();
        window.set_title(&format!("Maze Runner - {:.0} FPS", fps));
    }
}
