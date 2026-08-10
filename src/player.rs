use minifb::{Key, Window};
use nalgebra_glm::Vec2;
use std::f32::consts::PI;
use crate::maze::{is_wall, Maze};

pub struct Player {
    pub pos: Vec2,
    pub a: f32,
}

/*
* Procesa los eventos del teclado y mueve al jugador
* Parametros:
* window: ventana del juego
* player: jugador
* maze: laberinto
* block_size: tamaño de cada celda del laberinto
*/
pub fn process_events(window: &Window, player: &mut Player, maze: &Maze, block_size: usize) {
    //velocidad de movimiento
    const MOVE_SPEED: f32 = 10.0;
    //velocidad de rotacion
    const ROTATION_SPEED: f32 = PI / 10.0;

    if window.is_key_down(Key::A) {
        player.a -= ROTATION_SPEED;
    }

    if window.is_key_down(Key::D) {
        player.a += ROTATION_SPEED;
    }

    if window.is_key_down(Key::W) {
        let next_x = player.pos.x + MOVE_SPEED * player.a.cos();
        let next_y = player.pos.y + MOVE_SPEED * player.a.sin();

        if !is_wall(maze, next_x, player.pos.y, block_size) {
            player.pos.x = next_x;
        }

        if !is_wall(maze, player.pos.x, next_y, block_size) {
            player.pos.y = next_y;
        }
    }

    if window.is_key_down(Key::S) {
        let next_x = player.pos.x - MOVE_SPEED * player.a.cos();
        let next_y = player.pos.y - MOVE_SPEED * player.a.sin();

        if !is_wall(maze, next_x, player.pos.y, block_size) {
            player.pos.x = next_x;
        }

        if !is_wall(maze, player.pos.x, next_y, block_size) {
            player.pos.y = next_y;
        }
    }
}