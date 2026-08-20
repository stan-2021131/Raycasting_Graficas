mod caster;
mod framebuffer;
mod maze;
mod player;
mod line;
mod texture;
mod sprite;
mod render;

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::time::Duration;

use crate::framebuffer::Framebuffer;
use crate::maze::{load_maze, is_goal};
use crate::player::process_events;
use crate::texture::{get_texture, load_textures};
use crate::render::{render_2d, render_3d, render_minimap, render_sprite, BLOCK_SIZE};

fn main() {
    let window_width = 1000; // ancho de la ventana
    let window_height = 700; // alto de la ventana
    let framebuffer_width = 1300; // ancho del framebuffer
    let framebuffer_height = 900; // alto del framebuffer
    let frame_delay = Duration::from_millis(16); // delay entre frames
    let mut mode_3d = true; // modo 2d o 3d
    let mut last_mouse_x: Option<f32> = None; // última posición X del mouse
        // carga el laberinto una vez al inicio
    let (maze, mut player, goal) = load_maze("./maze.txt", BLOCK_SIZE);
    
    // cargamos las texturas una vez al inicio
    let textures = load_textures();
    let goal_texture = get_texture(&textures, 'g');

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    framebuffer.set_background_color(0x000000);

    let mut window = Window::new(
        "Maze Runner",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        //cambiar modo de renderizado
        if window.is_key_pressed(Key::M, KeyRepeat::No) {
            mode_3d = !mode_3d;
        }
        process_events(&mut window, &mut player, &maze, BLOCK_SIZE, &mut last_mouse_x);

        // ¿el jugador llegó a la meta? Se traduce su posición en píxeles a la
        // celda que ocupa y se revisa si esa celda es la marca `g`.
        if is_goal(&maze, player.pos.x, player.pos.y, BLOCK_SIZE) {
            println!("¡Meta alcanzada! Fin del juego.");
            break;
        }

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

        std::thread::sleep(frame_delay);
    }
}