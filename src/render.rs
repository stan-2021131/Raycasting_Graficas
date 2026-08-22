use std::f32::consts::PI;

use crate::caster::cast_ray;
use crate::framebuffer::Framebuffer;
use crate::line::line;
use crate::maze::Maze;
use crate::player::Player;
use crate::texture::{TextureCatalog, get_texture, Texture};
use crate::sprite::Sprite;

pub const BLOCK_SIZE: usize = 100;
pub const NUM_RAYS: usize = 5;
pub const FOV: f32 = PI / 3.0;

const MINIMAP_BLOCK_SIZE: usize = 15;

fn cell_color(cell: char) -> u32 {
    match cell {
        '+' => 0x00AAFF, // columnas
        '-' => 0xFF5555, // paredes horizontales
        '|' => 0xFF5555, // paredes verticales
        'g' | 'G' => 0x00FF00, // meta
        _ => 0x000000,   // cualquier otra cosa
    }
}

fn draw_cell(framebuffer: &mut Framebuffer, xo: usize, yo: usize, block_size: usize, cell: char) {
    if cell == ' ' {
        return;
    }

    framebuffer.set_current_color(cell_color(cell));

    for x in xo..xo + block_size {
        for y in yo..yo + block_size {
            framebuffer.point(x, y);
        }
    }
}

pub fn render_2d(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player) {
    let maze_cols = maze.first().map_or(0, |row| row.len());
    let maze_rows = maze.len();

    let block_size_x = if maze_cols > 0 { framebuffer.width / maze_cols } else { BLOCK_SIZE };
    let block_size_y = if maze_rows > 0 { framebuffer.height / maze_rows } else { BLOCK_SIZE };
    let draw_block_size = block_size_x.min(block_size_y);

    let scale = draw_block_size as f32 / BLOCK_SIZE as f32;

    for (row, line) in maze.iter().enumerate() {
        for (col, &cell) in line.iter().enumerate() {
            draw_cell(framebuffer, col * draw_block_size, row * draw_block_size, draw_block_size, cell);
        }
    }

    framebuffer.set_current_color(0xFFFF00);
    
    let px = (player.pos.x * scale) as usize;
    let py = (player.pos.y * scale) as usize;

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
            let hx = (hit_x * scale) as usize;
            let hy = (hit_y * scale) as usize;
            line(framebuffer, px, py, hx, hy);
        }
    }
}

pub fn render_minimap(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player) {
    // Calculamos el tamaño total del mapa para posicionarlo en la esquina superior derecha
    let maze_cols = maze.first().map_or(0, |row| row.len());
    let offset_x = framebuffer.width.saturating_sub(maze_cols * MINIMAP_BLOCK_SIZE + 20);
    let offset_y = 20;

    for (row, line) in maze.iter().enumerate() {
        for (col, &cell) in line.iter().enumerate() {
            draw_cell(framebuffer, offset_x + col * MINIMAP_BLOCK_SIZE, offset_y + row * MINIMAP_BLOCK_SIZE, MINIMAP_BLOCK_SIZE, cell);
        }
    }

    framebuffer.set_current_color(0xFFFF00);
    
    let scale = MINIMAP_BLOCK_SIZE as f32 / BLOCK_SIZE as f32;
    let px = offset_x + (player.pos.x * scale) as usize;
    let py = offset_y + (player.pos.y * scale) as usize;

    for x in px.saturating_sub(2)..=px + 2 {
        for y in py.saturating_sub(2)..=py + 2 {
            framebuffer.point(x, y);
        }
    }

    // Un rayo para la dirección de vista del jugador en el minimapa
    if let Some((_, _, hit_x, hit_y)) = cast_ray(maze, player, player.a, BLOCK_SIZE) {
        let hx = offset_x + (hit_x * scale) as usize;
        let hy = offset_y + (hit_y * scale) as usize;
        line(framebuffer, px, py, hx, hy);
    }
}

pub fn render_3d(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player, textures: &TextureCatalog) -> Vec<f32> {
    let mut z_buffer = vec![f32::INFINITY; framebuffer.width];

    let half_height = framebuffer.height as f32 / 2.0;
    let horizon_y = half_height as usize;

    // Colores para el cielo (arriba) y el suelo (abajo)
    let ceiling_color = 0x451206;
    let floor_color = 0x1A1615;  

    let projection_distance = (framebuffer.width as f32 / 2.0) / (FOV / 2.0).tan();
    let delta_beta = FOV / (framebuffer.width - 1) as f32;

    for i in 0..framebuffer.width {
        let beta = -FOV / 2.0 + delta_beta * i as f32;
        let ray_angle = player.a + beta;

        if let Some((distance, wall, hit_x, hit_y)) = cast_ray(maze, player, ray_angle, BLOCK_SIZE) {
            let corrected = distance * beta.cos();
            z_buffer[i] = corrected;

            let wall_height = (BLOCK_SIZE as f32 / corrected) * projection_distance;
            let real_top = half_height - wall_height / 2.0;
            let real_bottom = half_height + wall_height / 2.0;

            let top = (real_top.max(0.0) as usize).min(framebuffer.height);
            let bottom = (real_bottom.min(framebuffer.height as f32) as usize).max(top);

            // 1. Techo / Cielo (desde la parte superior hasta donde empieza la pared)
            framebuffer.set_current_color(ceiling_color);
            for y in 0..top {
                framebuffer.point(i, y);
            }

            // 2. Pared
            let local_x = hit_x % BLOCK_SIZE as f32;
            let local_y = hit_y % BLOCK_SIZE as f32;
            let bx = local_x.min(BLOCK_SIZE as f32 - local_x);
            let by = local_y.min(BLOCK_SIZE as f32 - local_y);
            let is_vertical = bx < by;

            let u = if is_vertical {
                local_y / BLOCK_SIZE as f32
            } else {
                local_x / BLOCK_SIZE as f32
            };

            let tex = get_texture(textures, wall);
            let tx = ((u * tex.width as f32) as usize).min(tex.width - 1);

            for y in top..bottom {
                let v = (y as f32 - real_top) / wall_height;
                let ty = ((v * tex.height as f32) as usize).min(tex.height - 1);
                let color = tex.get_pixel(tx, ty);

                framebuffer.set_current_color(color);
                framebuffer.point(i, y);
            }

            // 3. Suelo (desde donde termina la pared hasta el fondo de la pantalla)
            framebuffer.set_current_color(floor_color);
            for y in bottom..framebuffer.height {
                framebuffer.point(i, y);
            }
        } else {
            // Si el rayo no golpea nada, pintar directamente cielo y suelo divididos por el horizonte
            framebuffer.set_current_color(ceiling_color);
            for y in 0..horizon_y {
                framebuffer.point(i, y);
            }

            framebuffer.set_current_color(floor_color);
            for y in horizon_y..framebuffer.height {
                framebuffer.point(i, y);
            }
        }
    }
    z_buffer
}

pub fn render_sprite(
    framebuffer: &mut Framebuffer,
    player: &Player,
    sprite: &Sprite,
    texture: &Texture,
    z_buffer: &[f32],
) {
    const SPRITE_SIZE: f32 = BLOCK_SIZE as f32;
    const TRANSPARENT_COLOR: u32 = 0xD50BEB;

    // Vector jugador -> sprite
    // dx = sprite_x - jugador_x
    // dy = sprite_y - jugador_y
    let dx = sprite.pos.x - player.pos.x;
    let dy = sprite.pos.y - player.pos.y;

    // d = sqrt(dx² + dy²)
    let distance = (dx * dx + dy * dy).sqrt();

    // θ = atan2(dy, dx)
    let angle = dy.atan2(dx);

    // β = θ - a
    let mut beta = angle - player.a;

    // Normalizar β a [-PI, PI)
    beta = (beta + PI).rem_euclid(2.0 * PI) - PI;

    // Calcular el radio angular del sprite para culling
    let sprite_angular_half_width = (SPRITE_SIZE / 2.0 / distance).atan();

    // Descartar si el sprite está completamente fuera del FOV
    if beta - sprite_angular_half_width > FOV / 2.0 || beta + sprite_angular_half_width < -FOV / 2.0 {
        return;
    }

    // Δβ = FOV / (W - 1)
    let delta_beta = FOV / (framebuffer.width - 1) as f32;

    // i_centro = (β + FOV/2) / Δβ
    let screen_center = (beta + FOV / 2.0) / delta_beta;

    // corregida = d * cos(β)
    let corrected_distance = distance * beta.cos();

    // Evitar distancia prácticamente nula
    if corrected_distance <= 0.0 {
        return;
    }

    // d_plano = (W/2) / tan(FOV/2)
    let projection_distance = (framebuffer.width as f32 / 2.0) / (FOV / 2.0).tan();

    // lado =
    // (SPRITE_SIZE / corregida) * d_plano
    let sprite_size = (SPRITE_SIZE / corrected_distance) * projection_distance;

    // Horizonte = H/2
    let horizon = framebuffer.height as f32 / 2.0;

    // Orillas reales del sprite
    let left = screen_center - sprite_size / 2.0;

    let right = screen_center + sprite_size / 2.0;

    let top = horizon - sprite_size / 2.0;

    let bottom = horizon + sprite_size / 2.0;

    // Si el rectángulo entero quedó fuera
    if right < 0.0
        || left >= framebuffer.width as f32
        || bottom < 0.0
        || top >= framebuffer.height as f32
    {
        return;
    }

    // Recorte únicamente para pintar
    let draw_left = left.max(0.0) as usize;

    let draw_right = right.min(framebuffer.width as f32) as usize;

    let draw_top = top.max(0.0) as usize;

    let draw_bottom = bottom.min(framebuffer.height as f32) as usize;
        bottom.min(framebuffer.height as f32) as usize;

    // Recorrer el rectángulo del sprite
    for x in draw_left..draw_right {

        // Prueba de profundidad:
        // si la pared está más cerca, esta
        // columna del sprite queda oculta.
        if corrected_distance >= z_buffer[x] {
            continue;
        }

        // u = (x - izquierda) / lado
        let u = (x as f32 - left) / sprite_size;

        // tx = floor(u * Tw)
        let tx = ((u * texture.width as f32) as usize)
                .min(texture.width - 1);

        for y in draw_top..draw_bottom {

            // v = (y - arriba) / lado
            let v = (y as f32 - top) / sprite_size;

            // ty = floor(v * Th)
            let ty = ((v * texture.height as f32) as usize).min(texture.height - 1);

            // Obtener texel
            let color = texture.get_pixel(tx, ty);

            // Color clave = transparencia
            if color == TRANSPARENT_COLOR {
                continue;
            }

            framebuffer.set_current_color(color);
            framebuffer.point(x, y);
        }
    }
}

pub fn render_image(framebuffer: &mut Framebuffer, texture: &Texture) {
    for x in 0..framebuffer.width {
        let u = x as f32 / framebuffer.width as f32;
        let tx = ((u * texture.width as f32) as usize).min(texture.width - 1);

        for y in 0..framebuffer.height {
            let v = y as f32 / framebuffer.height as f32;
            let ty = ((v * texture.height as f32) as usize).min(texture.height - 1);

            let color = texture.get_pixel(tx, ty);
            framebuffer.set_current_color(color);
            framebuffer.point(x, y);
        }
    }
}
