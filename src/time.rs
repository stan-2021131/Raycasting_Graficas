use std::time::{Duration, Instant};
use crate::framebuffer::Framebuffer;
use crate::texture::Texture;

/// Maneja el tiempo regresivo de un nivel.
pub struct LevelTimer {
    pub time_left: Duration,
    last_update: Instant,
}

impl LevelTimer {
    /// Inicializa un temporizador con los segundos indicados.
    pub fn new(seconds: u64) -> Self {
        let duration = Duration::from_secs(seconds);
        Self {
            time_left: duration,
            last_update: Instant::now(),
        }
    }

    /// Resetea el reloj interno para que no reste el tiempo que pasó 
    /// mientras el juego estaba en el menú.
    pub fn reset_update_time(&mut self) {
        self.last_update = Instant::now();
    }

    /// Actualiza el tiempo restante restándole el tiempo real transcurrido.
    pub fn update(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update);
        self.last_update = now;

        if self.time_left > elapsed {
            self.time_left -= elapsed;
        } else {
            self.time_left = Duration::ZERO;
        }
    }

    /// Retorna true si el temporizador ha llegado a 00:00.
    pub fn is_finished(&self) -> bool {
        self.time_left.is_zero()
    }

    /// Dibuja el temporizador en formato MM:SS centrado horizontalmente en `center_x`.
    /// Recibe un arreglo de texturas donde los índices 0-9 son los números y el 10 es los dos puntos (:).
    pub fn render_time(
        &self, 
        framebuffer: &mut Framebuffer, 
        number_textures: &[Texture; 11], 
        center_x: usize, 
        y: usize,
        scale: usize
    ) {
        let secs = self.time_left.as_secs();
        let mins = secs / 60;
        let rem_secs = secs % 60;

        let m1 = (mins / 10) as usize;
        let m2 = (mins % 10) as usize;
        let s1 = (rem_secs / 10) as usize;
        let s2 = (rem_secs % 10) as usize;

        // Índices en el arreglo de texturas
        let digits = [m1, m2, 10, s1, s2];

        // Asumimos que todas las texturas de números tienen el mismo ancho base.
        let tex_width = number_textures[0].width as usize;
        let scaled_tex_width = tex_width * scale;
        let spacing = 2 * scale; // Espacio entre caracteres escalado

        let total_width = digits.len() * scaled_tex_width + (digits.len() - 1) * spacing;
        
        let mut current_x = center_x.saturating_sub(total_width / 2);

        for &digit in &digits {
            let tex = &number_textures[digit];
            
            for ty in 0..tex.height as usize {
                for tx in 0..tex.width as usize {
                    let color = tex.get_pixel(tx, ty);
                    
                    // El magenta (0xD50BEB) es el color transparente estándar del proyecto
                    if color != 0xD50BEB {
                        framebuffer.set_current_color(color);
                        // Dibujar el píxel escalado como un bloque
                        for sy in 0..scale {
                            for sx in 0..scale {
                                let px = current_x + tx * scale + sx;
                                let py = y + ty * scale + sy;
                                if px < framebuffer.width && py < framebuffer.height {
                                    framebuffer.point(px, py);
                                }
                            }
                        }
                    }
                }
            }

            current_x += scaled_tex_width + spacing;
        }
    }
}
