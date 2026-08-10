use crate::maze::{is_wall, Maze};
use crate::player::Player;


/**
 * Calcula la distancia desde la posición del jugador hasta la pared más cercana en la dirección `a`.
 */
pub fn cast_ray(
    maze: &Maze,
    player: &Player,
    a: f32,
    block_size: usize,
) -> f32 {
    let mut distance = 0.0;

    loop {
        let x = (player.pos.x + distance * a.cos()) as usize;
        let y = (player.pos.y + distance * a.sin()) as usize;

        if is_wall(maze, x as f32, y as f32, block_size) {
            return distance;
        }

        distance += 1.0;
    }
}