use minifb::{Key, MouseMode, Window};
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
pub fn process_events(window: &mut Window, player: &mut Player, maze: &Maze, block_size: usize, last_mouse_x: &mut Option<f32>) {
    //velocidad de movimiento
    const MOVE_SPEED: f32 = 10.0;
    //velocidad de rotacion
    const ROTATION_SPEED: f32 = PI / 10.0;
    //sensibilidad del mouse
    const MOUSE_SENSITIVITY: f32 = 0.005;

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

    // Movimiento relativo del mouse
    if let Some((mouse_x, _mouse_y)) =
        window.get_mouse_pos(MouseMode::Pass)
    {
        // Primera lectura:
        // solamente guardamos la posición.
        if let Some(previous_x) = *last_mouse_x {
            let delta_x = mouse_x - previous_x;

            // Si el mouse realmente se movió,
            // aplicamos la rotación.
            player.a += delta_x * MOUSE_SENSITIVITY;
        }

        // Guardamos la posición para el siguiente frame.
        *last_mouse_x = Some(mouse_x);
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