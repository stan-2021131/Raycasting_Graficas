use crate::maze::{get_cell_at, Maze};
use crate::player::Player;

pub fn cast_ray(
    maze: &Maze,
    player: &Player,
    angle: f32,
    block_size: usize,
) -> Option<(f32, char)> {
    let mut distance = 0.0;

    loop {
        let x = player.pos.x + distance * angle.cos();
        let y = player.pos.y + distance * angle.sin();
        match get_cell_at(maze, x, y, block_size) {
            Some(cell) if matches!(cell, '+' | '-' | '|' | 'g' | 'G') => {
                return Some((distance, cell));
            }
            Some(_) => {
                distance += 1.0;
            }
            None => {
                return None;
            }
        }
    }
}