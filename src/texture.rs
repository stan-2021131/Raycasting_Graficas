use std::collections::HashMap;

pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<u32>,
}

pub type TextureCatalog = HashMap<char, Texture>;

impl Texture {
    pub fn new(path: &str) -> Self {
        let image = image::open(path)
            .expect("No se pudo cargar la textura")
            .to_rgb8();

        let (width, height) = image.dimensions();

        let buffer = image
            .pixels()
            .map(|pixel| {
                let [r, g, b] = pixel.0;

                ((r as u32) << 16)
                    | ((g as u32) << 8)
                    | b as u32
            })
            .collect();

        Self {
            width: width as usize,
            height: height as usize,
            buffer,
        }
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> u32 {
        self.buffer[y * self.width + x]
    }
}

pub fn load_textures() -> TextureCatalog {
    let mut textures = TextureCatalog::new();

    textures.insert(
        '+',
        Texture::new("./textures/column.png"),
    );

    textures.insert(
        '-',
        Texture::new("./textures/wall.png"),
    );

    textures.insert(
        '|',
        Texture::new("./textures/wall.png"),
    );
    textures.insert(
        'g',
        Texture::new("./textures/wall.png"),
    );

    textures
}

pub fn get_texture<'a>(
    textures: &'a TextureCatalog,
    cell: char,
) -> &'a Texture {
    textures
        .get(&cell)
        .expect("No existe textura para esta celda")
}