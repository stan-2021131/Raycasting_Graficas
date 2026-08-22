use nalgebra_glm::Vec2;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use crate::maze::{Maze, is_wall};
use crate::player::Player;

const PLAYER_RADIUS: f32 = 20.0;
const ENEMY_RADIUS: f32 = 20.0;

/// Estructura que representa a un enemigo (Zombie).
#[derive(Clone)]
pub struct Enemy {
    /// Posición en coordenadas de mundo.
    pub pos: Vec2,
    /// Velocidad de movimiento por frame.
    pub speed: f32,
    /// Ruta calculada por A*, como lista de puntos de mundo a los que dirigirse.
    pub path: Vec<Vec2>,
    /// Siguiente punto objetivo dentro de `path`.
    pub path_index: usize,
    /// Offset para desfasar la animación y que no todos los zombies muevan los pies al unísono.
    pub animation_offset: u128,
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct State {
    cost: i32,
    position: (usize, usize),
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Invertimos el orden para crear un Min-Heap en lugar de Max-Heap
        other.cost.cmp(&self.cost) 
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Enemy {
    /// Crea un nuevo enemigo en la posición especificada y con una velocidad determinada.
    pub fn new(x: f32, y: f32, speed: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
            speed,
            path: Vec::new(),
            path_index: 0,
            animation_offset: (x as u128 + y as u128) % 1000, // Pseudo-aleatorio según pos inicial
        }
    }

    /// Mueve al enemigo un paso a través de su ruta calculada.
    pub fn update(&mut self) {
        if self.path_index >= self.path.len() {
            return; // Llegó al final de su ruta o no tiene ruta.
        }

        let target = self.path[self.path_index];
        let dx = target.x - self.pos.x;
        let dy = target.y - self.pos.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance <= self.speed {
            // Alcanzó el punto exacto (o se pasaría), se ajusta y avanza de índice
            self.pos = target;
            self.path_index += 1;
        } else {
            // Movimiento progresivo en dirección al target
            let dir_x = dx / distance;
            let dir_y = dy / distance;
            self.pos.x += dir_x * self.speed;
            self.pos.y += dir_y * self.speed;
        }
    }

    /// Recalcula la ruta del enemigo al jugador usando A*.
    pub fn recalculate_path(&mut self, maze: &Maze, block_size: usize, player: &Player) {
        let start_col = (self.pos.x / block_size as f32) as usize;
        let start_row = (self.pos.y / block_size as f32) as usize;
        let goal_col = (player.pos.x / block_size as f32) as usize;
        let goal_row = (player.pos.y / block_size as f32) as usize;

        // Si ya está en la misma celda, se limpia la ruta
        if start_col == goal_col && start_row == goal_row {
            self.path.clear();
            self.path_index = 0;
            return;
        }

        if let Some(path) = a_star_path(maze, block_size, (start_col, start_row), (goal_col, goal_row)) {
            self.path = path;
            self.path_index = 0;
        }
    }
}

/// Comprueba si hubo colisión entre el enemigo y el jugador, utilizando suma de radios.
pub fn check_collision(enemy: &Enemy, player: &Player) -> bool {
    let dx = enemy.pos.x - player.pos.x;
    let dy = enemy.pos.y - player.pos.y;
    let dist_sq = dx * dx + dy * dy;
    let min_dist = PLAYER_RADIUS + ENEMY_RADIUS;
    dist_sq <= min_dist * min_dist
}

/// Algoritmo A* para encontrar la ruta más corta en el laberinto
fn a_star_path(maze: &Maze, block_size: usize, start: (usize, usize), goal: (usize, usize)) -> Option<Vec<Vec2>> {
    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
    let mut g_score: HashMap<(usize, usize), i32> = HashMap::new();

    g_score.insert(start, 0);
    open_set.push(State { cost: heuristic(start, goal), position: start });

    let maze_height = maze.len();
    let maze_width = maze.first().map_or(0, |r| r.len());

    while let Some(State { cost: _, position }) = open_set.pop() {
        if position == goal {
            return Some(reconstruct_path(&came_from, position, block_size));
        }

        let current_g = *g_score.get(&position).unwrap_or(&i32::MAX);

        // Movimientos ortogonales
        let neighbors = [
            (position.0 as i32 + 1, position.1 as i32),
            (position.0 as i32 - 1, position.1 as i32),
            (position.0 as i32, position.1 as i32 + 1),
            (position.0 as i32, position.1 as i32 - 1),
        ];

        for &(nc, nr) in &neighbors {
            if nc < 0 || nr < 0 { continue; }
            let nc = nc as usize;
            let nr = nr as usize;

            if nr >= maze_height || nc >= maze_width { continue; }

            let cell_x = (nc * block_size + block_size / 2) as f32;
            let cell_y = (nr * block_size + block_size / 2) as f32;
            
            // Si es pared, es intransitable
            if is_wall(maze, cell_x, cell_y, block_size) {
                continue;
            }

            let tentative_g = current_g + 1; // costo unitario por moverse entre celdas adyacentes
            let neighbor_g = *g_score.get(&(nc, nr)).unwrap_or(&i32::MAX);

            if tentative_g < neighbor_g {
                came_from.insert((nc, nr), position);
                g_score.insert((nc, nr), tentative_g);
                let f_score = tentative_g + heuristic((nc, nr), goal);
                open_set.push(State { cost: f_score, position: (nc, nr) });
            }
        }
    }

    // No se encontró ruta posible
    None
}

/// Heurística Manhattan para A*
fn heuristic(a: (usize, usize), b: (usize, usize)) -> i32 {
    (a.0 as i32 - b.0 as i32).abs() + (a.1 as i32 - b.1 as i32).abs()
}

/// Reconstruye el camino desde `goal` al `start` siguiendo los registros `came_from`.
fn reconstruct_path(came_from: &HashMap<(usize, usize), (usize, usize)>, mut current: (usize, usize), block_size: usize) -> Vec<Vec2> {
    let mut total_path = Vec::new();
    
    // Se inserta el nodo final
    total_path.push(Vec2::new(
        (current.0 * block_size + block_size / 2) as f32,
        (current.1 * block_size + block_size / 2) as f32,
    ));

    // Deshacer el camino hasta el origen
    while let Some(&prev) = came_from.get(&current) {
        current = prev;
        total_path.push(Vec2::new(
            (current.0 * block_size + block_size / 2) as f32,
            (current.1 * block_size + block_size / 2) as f32,
        ));
    }

    // Invertir para que vaya del origen al destino
    total_path.reverse();
    
    // Descartamos la celda de inicio ya que el zombie actualmente se encuentra en ella
    if total_path.len() > 1 {
        total_path.remove(0);
    }
    
    total_path
}
