mod caster;
mod framebuffer;
mod maze;
mod player;
mod line;

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::f32::consts::PI;
use std::time::Duration;

use crate::caster::cast_ray;
use crate::framebuffer::Framebuffer;
use crate::line::line;
use crate::maze::{load_maze, is_goal, Maze};
use crate::player::{process_events, Player};

const BLOCK_SIZE: usize = 100;

/// Cantidad de rayos que se lanzan en abanico para formar el campo de visión.
const NUM_RAYS: usize = 5;

/// Amplitud del campo de visión (field of view), en radianes.
const FOV: f32 = PI / 3.0;

fn cell_color(cell: char) -> u32 {
    match cell {
        '+' => 0x00AAFF, // columnas
        '-' => 0xFF5555, // paredes horizontales
        '|' => 0xFF5555, // paredes verticales
        'g' | 'G' => 0x00FF00, // meta
        _ => 0xFFDDDD,   // cualquier otra cosa
    }
}

fn draw_cell(framebuffer: &mut Framebuffer, xo: usize, yo: usize, cell: char) {
    if cell == ' ' {
        return;
    }

    framebuffer.set_current_color(cell_color(cell));

    for x in xo..xo + BLOCK_SIZE {
        for y in yo..yo + BLOCK_SIZE {
            framebuffer.point(x, y);
        }
    }
}

fn render_2d(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player) {
    for (row, line) in maze.iter().enumerate() {
        for (col, &cell) in line.iter().enumerate() {
            draw_cell(framebuffer, col * BLOCK_SIZE, row * BLOCK_SIZE, cell);
        }
    }

    framebuffer.set_current_color(0xFFFF00);
    
    let px = player.pos.x as usize;
    let py = player.pos.y as usize;

    for x in px.saturating_sub(3)..=px + 3 {
        for y in py.saturating_sub(3)..=py + 3 {
            framebuffer.point(x, y);
        }
    }

    // lanza un abanico de rayos centrado en la dirección de vista del jugador.
    // El campo de visión (FOV) se reparte de forma pareja entre los NUM_RAYS
    // rayos: el primero apunta a `a - FOV/2`, el último a `a + FOV/2` y el del
    // medio coincide con la dirección de vista.
    for i in 0..NUM_RAYS {
        let ray_fraction = i as f32 / (NUM_RAYS - 1) as f32; // de 0.0 a 1.0
        let angle = player.a - FOV / 2.0 + FOV * ray_fraction;

        if let Some((distance, _)) = cast_ray(maze, player, angle, BLOCK_SIZE) {
            let end_x = player.pos.x + distance * angle.cos();
            let end_y = player.pos.y + distance * angle.sin();
            line(framebuffer, player.pos.x as usize, player.pos.y as usize, end_x as usize, end_y as usize);
        }
    }
}

fn render_3d(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player) {
    // Mitad de la altura de la pantalla para centrar la proyección.
    let half_height = framebuffer.height as f32 / 2.0;

    // Distancia de proyección, que define el campo de visión.
    let projection_distance =
        (framebuffer.width as f32 / 2.0)
        / (FOV / 2.0).tan();

    // Incremento angular entre rayos adyacentes.
    let delta_beta = FOV / (framebuffer.width - 1) as f32;
    
        for i in 0..framebuffer.width {
        // βi = -FOV/2 + Δβ * i
        let beta =
            -FOV / 2.0
            + delta_beta * i as f32;

        // θ = a + βi
        let ray_angle =
            player.a + beta;

        if let Some((distance, wall)) = cast_ray(maze, player, ray_angle, BLOCK_SIZE) {
            // Evita la distorsión de la "fish-eye" corrigiendo la distancia proyectada.
            let corrected = distance * beta.cos();

            // Altura de la pared en la pantalla.
            let wall_height = (BLOCK_SIZE as f32 / corrected) * projection_distance;
            // Coordenadas verticales de la porción de pared a dibujar.
            let top =
                (half_height - wall_height / 2.0).max(0.0);
            let bottom =
                (half_height + wall_height / 2.0).min(framebuffer.height as f32);

            // Color de la pared golpeada
            framebuffer.set_current_color(cell_color(wall));

            // Dibujar la estaca
            line(framebuffer, i, top as usize, i, bottom as usize);
        }
    }
}

fn main() {
    let window_width = 1000;
    let window_height = 700;
    let framebuffer_width = 1300;
    let framebuffer_height = 900;
    let frame_delay = Duration::from_millis(16);
    let mut mode_3d = false;

    let (maze, mut player) = load_maze("./maze.txt", BLOCK_SIZE);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    framebuffer.set_background_color(0x333355);

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
        process_events(&window, &mut player, &maze, BLOCK_SIZE);

        // ¿el jugador llegó a la meta? Se traduce su posición en píxeles a la
        // celda que ocupa y se revisa si esa celda es la marca `g`.
        if is_goal(&maze, player.pos.x, player.pos.y, BLOCK_SIZE) {
            println!("¡Meta alcanzada! Fin del juego.");
            break;
        }

        framebuffer.clear();

        if mode_3d {
            render_3d(&mut framebuffer, &maze, &player);
        } else {
            render_2d(&mut framebuffer, &maze, &player);
        }

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}