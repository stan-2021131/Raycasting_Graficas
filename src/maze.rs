use std::f32::consts::PI;
use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::sprite::Sprite;

use nalgebra_glm::Vec2;

use crate::player::Player;

pub type Maze = Vec<Vec<char>>;

pub fn load_maze(filename: &str, block_size: usize) -> (Maze, Player, Sprite) {
    let file = File::open(filename).expect("no se pudo abrir el archivo del laberinto");

    let reader = BufReader::new(file);

    let mut maze: Maze = Vec::new();

    let mut player_pos: Option<Vec2> = None;
    let mut goal_pos: Option<Vec2> = None;

    for (row, line) in reader.lines().enumerate() {
        let line = line.expect("no se pudo leer una línea del laberinto");

        let mut cells: Vec<char> = Vec::new();

        for (col, character) in line.chars().enumerate() {
            if character == 'p' {
                let x = col * block_size + block_size / 2;
                let y = row * block_size + block_size / 2;
                player_pos = Some(Vec2::new(x as f32, y as f32));

                cells.push(' ');
            } else if character == 'g' || character == 'G' {
                let x = col * block_size + block_size / 2;
                let y = row * block_size + block_size / 2;
                goal_pos = Some(Vec2::new(x as f32, y as f32));

                cells.push(character);
            }
            else {
                cells.push(character);
            }
        }

        maze.push(cells);
    }

    let player = Player {
        pos: player_pos.unwrap_or_else(|| Vec2::new(0.0, 0.0)),
        // ángulo de vista inicial; el jugador podrá girarlo con el teclado.
        a: PI / 3.0,
    };

    let pos_goal = goal_pos.unwrap_or_else(|| Vec2::new(0.0, 0.0));
    let goal = Sprite::new(pos_goal.x, pos_goal.y);

    (maze, player, goal)
}

/// Obtiene la celda correspondiente a una posición del mundo.
pub fn get_cell_at(
    maze: &Maze,
    x: f32,
    y: f32,
    block_size: usize,
) -> Option<char> {
    // Coordenadas negativas están fuera del mapa
    if x < 0.0 || y < 0.0 {
        return None;
    }

    // Convertir coordenadas del mundo a coordenadas del maze
    let col = x as usize / block_size;
    let row = y as usize / block_size;

    maze.get(row)
        .and_then(|line| line.get(col))
        .copied()
}

/// Indica si una posición corresponde a una pared.
pub fn is_wall(
    maze: &Maze,
    x: f32,
    y: f32,
    block_size: usize,
) -> bool {
    match get_cell_at(maze, x, y, block_size) {
        Some('+' | '-' | '|') => true,

        // Cualquier otra celda es caminable
        Some(_) => false,

        // Fuera del mapa se considera pared
        None => true,
    }
}


/// Indica si una posición corresponde a la meta.
pub fn is_goal(
    maze: &Maze,
    x: f32,
    y: f32,
    block_size: usize,
) -> bool {
    matches!(
        get_cell_at(maze, x, y, block_size),
        Some('g' | 'G')
    )
}