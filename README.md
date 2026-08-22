# Raycasting Graficas

## Video Demostrativo del Juego

<div align="center">

<video src="./demo/demo.mp4"></video>

</div>

---

## Descripción General

**00:00** es un juego que utiliza *ray‑casting* escrito en **Rust** que combina renderizado **2D** y **3D** para la exploración de laberintos. El jugador controla a un personaje en primera persona con la capacidad de desplazarse y rotar su punto de vista, esquivar y huir de enemigos tipo *zombies* guiados por el algoritmo de búsqueda de caminos **A\*** (A‑estrella), y encontrar la meta antes de que el temporizador regresivo de nivel llegue a cero.

El proyecto cuenta con las siguientes características:
- Renderizado de muros 3D texturizados proyectados a partir del cálculo de distancia de los rayos lanzados desde el jugador.
- Renderizado alternativo 2D cenital (vista de mapa de cuadrícula).
- Sprites en formato *billboard* auto-orientados (meta y enemigos animándose secuencialmente).
- Sistema de oclusión/profundidad mediante **Z-buffer** para ocultar sprites de forma correcta detrás de muros.
- Audio ambiental en bucle y efectos de pasos sincronizados mediante la biblioteca **rodio**.
- Un temporizador regresivo basado en `Instant`/`Duration` que escala sus dígitos usando texturas numéricas.

---

## Estructura del Código y Funcionalidad de Archivos

### 1. [`src/main.rs`](/src/main.rs)
Es el punto de entrada de la aplicación.
- **Funcionalidades principales**:
  - Implementa el game loop con control de FPS.
  - Gestiona la máquina de estados del juego: `Menu`, `Controls`, `Playing`, `Completed`, `Lost`.
  - Coordina la carga de texturas, sprites, inicialización de audio (`AudioPlayer`) y del temporizador.
  - Ejecuta la lógica central de colisiones y transiciones de estados del juego.

### 2. [`src/player.rs`](/src/player.rs)
Modela el estado del jugador en el laberinto.
- **Funcionalidades principales**:
  - Contiene la estructura `Player` con su posición en coordenadas continuas y el ángulo de visión.
  - Procesa los inputs del teclado/ratón en `process_events` y actualiza la posición y rotación aplicando detección elemental de colisiones contra los límites de las celdas del laberinto.

### 3. [`src/maze.rs`](/src/maze.rs)
Responsable del almacenamiento de los datos estructurales del mapa y su inicialización.
- **Funcionalidades principales**:
  - Carga laberintos desde archivos de texto plano (`load_maze`).
  - Mapea caracteres del archivo a celdas ocupadas o vacías en la estructura `Maze`.
  - Descubre y parsea la posición inicial del jugador (`p`), la meta (`g`), y los zombies (`z`), removiéndolos de la cuadrícula física para dejarlos como entidades dinámicas libres.

### 4. [`src/render.rs`](/src/render.rs)
Contiene todo el motor gráfico de dibujo píxel a píxel sobre el Framebuffer.
- **Funcionalidades principales**:
  - `render_2d`: Dibuja el laberinto visto desde arriba y al jugador.
  - `render_3d`: Lanza una cantidad fija de rayos (`NUM_RAYS`) abarcando el Campo de Visión (FOV). Calcula intersecciones con muros, aplica corrección de distorsión de lente de ojo de pez y dibuja las columnas de píxeles proyectadas rellenando el **Z-buffer** de distancia.
  - `render_sprite`: Renderiza entidades 3D (meta y zombies) escalándolas según la distancia, aplicando el test del Z-buffer por cada columna vertical dibujada para que los muros oculten correctamente los elementos que quedan detrás.
  - Contiene funciones utilitarias de dibujo para pantallas estáticas, minimapa y control de buffers.

### 5. [`src/enemy.rs`](/src/enemy.rs)
Encapsula el comportamiento físico y cognitivo de los enemigos.
- **Funcionalidades principales**:
  - Define la estructura `Enemy` (zombie) que posee posición, velocidad (75% respecto a la del jugador), ruta actual y desfase de animación.
  - Implementa el algoritmo de pathfinding **A\*** (`a_star_path`) que evalúa nodos en una cuadrícula lógica con una heurística de Manhattan para guiar al zombie hacia la celda del jugador.
  - `update`: Mueve al zombie a través de los nodos de la ruta calculada.
  - `check_collision`: Detecta si la distancia euclidiana entre el jugador y cualquier zombie es inferior a un umbral predefinido de peligro, activando el estado de derrota.

### 6. [`src/time.rs`](/src/time.rs)
Encargado de la lógica temporal de los niveles.
- **Funcionalidades principales**:
  - Define `LevelTimer` para el control de tiempo real mediante la medición del tiempo transcurrido desde el último frame usando `Instant`.
  - Expone el método `render_time` que descompone los segundos restantes en caracteres de texto (`MM:SS`) y dibuja las texturas numéricas correspondientes en el framebuffer a partir de un parámetro de escala personalizado.

### 7. [`src/sound.rs`](/src/sound.rs)
Centraliza la administración del sonido con la librería **rodio**.
- **Funcionalidades principales**:
  - Carga en memoria una sola vez al inicio del programa las pistas de audio para evitar accesos repetitivos a disco.
  - Reproduce en bucle la música de fondo.
  - Gestiona la cola y la reproducción periódica del efecto de sonido de pasos (`footsteps`) cuando la posición lógica del jugador cambia.

### 8. [`src/texture.rs`](/src/texture.rs)
Abstracción de texturas.
- **Funcionalidades principales**:
  - Lee imágenes en formato PNG y almacena la paleta de colores RGB de cada píxel.
  - Ofrece la función `get_pixel` para obtener la información de color dada una coordenada U,V.

---



## Instrucciones de Ejecución

### Requisitos Previos
- Tener instalado el compilador de **Rust** (`rustc` y `cargo`). Puedes descargarlo desde [rustup.rs](https://rustup.rs/).
- Librerías de desarrollo del sistema de audio de tu sistema operativo (si corresponde).

### Pasos
1. **Compilar el proyecto**:
   ```bash
   cargo build --release
   ```
2. **Ejecutar el programa**:
   ```bash
   cargo run
   ```

### Controles en el Juego
- **W / S** (en el menú): Subir / Bajar la selección de dificultad.
- **Enter** (en el menú): Iniciar nivel con la dificultad seleccionada.
- **W / A / S / D** o **Flechas Direccionales**: Mover al jugador y girar la cámara dentro del laberinto.
- **M**: Alternar dinámicamente entre la perspectiva 3D por ray-casting y el mapa cenital 2D.
- **Escape**: Regresar al menú principal.

