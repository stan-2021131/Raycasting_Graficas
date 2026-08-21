use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::fs::File;
use std::io::{Cursor, Read};
use std::time::{Duration, Instant};

pub struct AudioPlayer {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    music_sink: Sink,
    music_data: Option<Vec<u8>>,
    footstep_data: Option<Vec<u8>>,
    last_footstep: Instant,
}

impl AudioPlayer {
    pub fn new() -> Self {
        // Inicialización de la salida de audio
        let (_stream, stream_handle) = OutputStream::try_default().expect("Fallo la inicialización de la salida de audio");
        let music_sink = Sink::try_new(&stream_handle).expect("Fallo la creación del sink de audio");
        
        // Cargamos los archivos en memoria
        let music_data = Self::load_file_to_memory("./assets/music.ogg");
        let footstep_data = Self::load_file_to_memory("./assets/footstep.ogg");
        
        AudioPlayer {
            _stream,
            stream_handle,
            music_sink,
            music_data,
            footstep_data,
            last_footstep: Instant::now() - Duration::from_secs(10), // Inicializamos de forma que el cooldown ya haya pasado
        }
    }
    
    fn load_file_to_memory(path: &str) -> Option<Vec<u8>> {
        match File::open(path) {
            Ok(mut file) => {
                let mut data = Vec::new();
                if let Ok(_) = file.read_to_end(&mut data) {
                    Some(data)
                } else {
                    println!("Advertencia: No se pudo leer el contenido de {}", path);
                    None
                }
            }
            Err(_e) => {
                println!("Error: No se pudo abrir el archivo {}", path);
                None
            }
        }
    }
    
    pub fn start_music(&self) {
        if self.music_sink.empty() {
            if let Some(data) = &self.music_data {
                let cursor = Cursor::new(data.clone());
                if let Ok(decoder) = Decoder::new(cursor) {
                    self.music_sink.append(decoder.repeat_infinite());
                    self.music_sink.play();
                } else {
                    println!("Falló la decodificación de los datos de audio de la música.");
                }
            }
        } else if self.music_sink.is_paused() {
            self.music_sink.play();
        }
    }
    
    pub fn stop_music(&self) {
        self.music_sink.pause();
        self.music_sink.clear();
    }
    
    pub fn try_play_footstep(&mut self) {
        if self.last_footstep.elapsed() > Duration::from_millis(400) {
            if let Some(data) = &self.footstep_data {
                let cursor = Cursor::new(data.clone());
                if let Ok(decoder) = Decoder::new(cursor) {
                    if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                        sink.append(decoder);
                        sink.detach(); // Reproducir en segundo plano y limpiar automáticamente al terminar
                        self.last_footstep = Instant::now();
                    }
                }
            }
        }
    }
}
