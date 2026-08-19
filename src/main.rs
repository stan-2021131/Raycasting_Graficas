mod caster;
mod framebuffer;
mod maze;
mod player;
mod line;
mod texture;

use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::f32::consts::PI;
use std::time::Duration;

use crate::caster::cast_ray;
use crate::framebuffer::Framebuffer;
use crate::line::line;
use crate::maze::{load_maze, is_goal, Maze};
use crate::player::{process_events, Player};
use crate::texture::{TextureCatalog, get_texture, load_textures};

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
        _ => 0x000000,   // cualquier otra cosa
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

        if let Some((_, _, hit_x, hit_y)) = cast_ray(maze, player, angle, BLOCK_SIZE) {
            line(framebuffer, player.pos.x as usize, player.pos.y as usize, hit_x as usize, hit_y as usize);
        }
    }
}

fn render_3d(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player, textures: &TextureCatalog) {
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

        if let Some((distance, wall, hit_x, hit_y)) = cast_ray(maze, player, ray_angle, BLOCK_SIZE) {
            // Evita la distorsión de la "fish-eye" corrigiendo la distancia proyectada.
            let corrected = distance * beta.cos();

            // Altura de la pared en la pantalla.
            let wall_height = (BLOCK_SIZE as f32 / corrected) * projection_distance;
            // Orillas reales de la pared.
            let real_top =
                half_height - wall_height / 2.0;
            let real_bottom =
                half_height + wall_height / 2.0;

            // Orillas que se dibujarán.
            let top = real_top.max(0.0);
            let bottom = real_bottom.min(framebuffer.height as f32);

            // Posicion dentro del bloque
            let local_x = hit_x % BLOCK_SIZE as f32;
            let local_y = hit_y % BLOCK_SIZE as f32;

            // Distancia desde donde entra al bloque
            let bx = local_x.min(BLOCK_SIZE as f32 - local_x);
            let by = local_y.min(BLOCK_SIZE as f32 - local_y);

            // si bx < by, la pared es vertical
            let is_vertical = bx < by;

            let u = if is_vertical {
                local_y / BLOCK_SIZE as f32
            } else {
                local_x / BLOCK_SIZE as f32
            };

            // Obtener la textura correspondiente a la celda golpeada.
            let tex = get_texture(textures, wall);

            let tx =
                ((u * tex.width as f32) as usize)
                .min(tex.width - 1);
            
            // Recorrer verticalmente la estaca
            for y in top as usize..bottom as usize {
                // v = (y - arriba) / altura
                let v =
                    (y as f32 - real_top)
                    / wall_height;

                // v → ty
                let ty =
                    ((v * tex.height as f32) as usize)
                        .min(tex.height - 1);

                // Leer texel
                let color =
                    tex.get_pixel(tx, ty);

                // Dibujar texel
                framebuffer.set_current_color(color);
                framebuffer.point(i, y);
            }
        }
    }
}

fn main() {
    let window_width = 1000; // ancho de la ventana
    let window_height = 700; // alto de la ventana
    let framebuffer_width = 1300; // ancho del framebuffer
    let framebuffer_height = 900; // alto del framebuffer
    let frame_delay = Duration::from_millis(16); // delay entre frames
    let mut mode_3d = false; // modo 2d o 3d
    let mut last_mouse_x: Option<f32> = None; // última posición X del mouse
        // carga el laberinto una vez al inicio
    let (maze, mut player) = load_maze("./maze.txt", BLOCK_SIZE);
    
    // cargamos las texturas una vez al inicio
    let textures = load_textures();

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
            render_3d(&mut framebuffer, &maze, &player, &textures);
        } else {
            render_2d(&mut framebuffer, &maze, &player);
        }

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}