use nalgebra_glm::Vec2;

pub struct Sprite {
    pub pos: Vec2,
}

impl Sprite {
    pub fn new(x: f32, y: f32) -> Self {
        Sprite {
            pos: Vec2::new(x, y),
        }
    }
}