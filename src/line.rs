use crate::framebuffer::Framebuffer;

/**
 * Dibuja una línea en el framebuffer
 * Parametros:
 * framebuffer: framebuffer en el que se dibujará la línea
 * x1, y1: coordenadas del primer punto
 * x2, y2: coordenadas del segundo punto
 */
pub fn line(
    framebuffer: &mut Framebuffer,
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
) {
    let dx = (x2 as i32 - x1 as i32).abs();
    let dy = -(y2 as i32 - y1 as i32).abs();

    let sx = if x1 < x2 { 1 } else { -1 };
    let sy = if y1 < y2 { 1 } else { -1 };

    let mut err = dx + dy;

    let mut x = x1 as i32;
    let mut y = y1 as i32;

    loop {
        framebuffer.point(x as usize, y as usize);

        if x == x2 as i32 && y == y2 as i32 {
            break;
        }

        let e2 = 2 * err;

        if e2 >= dy {
            err += dy;
            x += sx;
        }

        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}