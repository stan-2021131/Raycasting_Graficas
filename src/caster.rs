use crate::maze::{get_cell_at, Maze};
use crate::player::Player;

pub fn cast_ray(
    maze: &Maze,
    player: &Player,
    angle: f32,
    block_size: usize,
) -> Option<(f32, char, f32, f32)> {
    const EPSILON: f32 = 0.001;

    let dir_x = angle.cos();
    let dir_y = angle.sin();

    let block = block_size as f32;

    let mut distance = 0.0;

    loop {
        let current_x = player.pos.x + distance * dir_x;
        let current_y = player.pos.y + distance * dir_y;

        // Siguiente frontera vertical
        let next_x = if dir_x > 0.0 {
            ((current_x / block).floor() + 1.0) * block
        } else {
            ((current_x / block).ceil() - 1.0) * block
        };

        // Siguiente frontera horizontal
        let next_y = if dir_y > 0.0 {
            ((current_y / block).floor() + 1.0) * block
        } else {
            ((current_y / block).ceil() - 1.0) * block
        };

        // Distancia hasta cada frontera
        let distance_x = if dir_x.abs() > EPSILON {
            (next_x - player.pos.x) / dir_x
        } else {
            f32::INFINITY
        };

        let distance_y = if dir_y.abs() > EPSILON {
            (next_y - player.pos.y) / dir_y
        } else {
            f32::INFINITY
        };

        // La frontera más cercana
        distance = distance_x.min(distance_y);

        // Punto EXACTO de impacto
        let hit_x = player.pos.x + distance * dir_x;
        let hit_y = player.pos.y + distance * dir_y;

        // Avanzar mínimamente para consultar
        // la celda al otro lado de la frontera
        let check_x = hit_x + dir_x * EPSILON;
        let check_y = hit_y + dir_y * EPSILON;

        match get_cell_at(maze, check_x, check_y, block_size) {
            Some(cell) if matches!(cell, '+' | '-' | '|') => {
                return Some((distance, cell, hit_x, hit_y));
            }

            Some(_) => {}

            None => {
                return None;
            }
        }

        // Evitar volver a calcular exactamente
        // la misma frontera
        distance += EPSILON;
    }
}