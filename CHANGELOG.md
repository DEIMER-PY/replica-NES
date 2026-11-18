# Changelog

Todos los cambios relevantes del proyecto se registran aquí.
El formato sigue [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) y el proyecto usa [versionado semántico](https://semver.org/lang/es/).

## [Sin publicar]

## [0.3.0] - 2026-11-18

Icono propio, más opciones de imagen, ejecutable de Linux compatible con más distribuciones y documentación con juegos probados.

### Añadido

- Documentación: capturas con juegos, registro de pruebas en Windows y Linux (`docs/PRUEBAS.md`), tabla de tecnologías y diagramas de componentes, del ciclo de un frame y de la publicación.
- Opciones de imagen: estirar hasta llenar la ventana, suavizado, líneas de barrido (efecto de televisor de tubo) y giro de 90°, 180° y 270°.
- Icono propio de la app: en la ventana, en el ejecutable de Windows (Explorador y barra de tareas) y como favicon de la versión web.

### Corregido

- El ejecutable de Linux de la v0.2.0 necesitaba glibc 2.43 y no abría en la mayoría de distribuciones; ahora se compila en Ubuntu 22.04 y funciona desde glibc 2.35 (Ubuntu 22.04, Debian 12, Fedora 36 y posteriores).

### Cambiado

- CI: antes de publicar, cada ejecutable se abre 15 segundos con la ROM de demostración (en Linux con un servidor X virtual) y se guarda una captura.

## [0.2.0] - 2026-11-16

Pantalla de inicio con biblioteca, botones de mando reasignables, más rendimiento y ejecutables descargables.

### Añadido

- `nes-app`: pantalla de inicio con botones para abrir una ROM, biblioteca de la carpeta de juegos, recientes y aviso al arrastrar un archivo.
- `nes-app`: reasignación de los botones del mando por jugador en **Ajustes → Controles**, también para botones que el sistema no reconoce (mandos genéricos).

- `docs/INSTALACION.md`: instalación por sistema operativo, requisitos mínimos y dónde conseguir ROMs.
- `nes-core`: `skip_frame` y `run_frame_silent`, que emulan un frame sin generar imagen o sonido.
- `nes-core`: ejemplo `bench` para medir la velocidad de emulación.
- CI: workflow `release` que compila los ejecutables de Windows y Linux y los adjunta a la release al subir un tag.

### Cambiado

- `nes-app`: sin ROM en la línea de comandos, la app arranca en la pantalla de inicio en lugar de cargar la demo.
- `nes-app`: la ventana de ajustes se desplaza si no cabe en la ventana de la app.
- Perfil `release` con LTO completo y `panic = "abort"`: un 16 % más rápido en el benchmark, y ejecutable sin símbolos.
- `nes-app`: el avance rápido solo dibuja el último frame de cada repintado y no genera audio.
- `nes-app`: en pausa o sin juego la ventana deja de repintarse de forma continua, para no ocupar la CPU ni la gráfica.

## [0.1.0] - 2026-11-16

Primera versión: emulador jugable en escritorio y en el navegador.

### Añadido

- Estructura del workspace con los crates `nes-core` y `nes-app`.
- Documentación base, guía de desarrollo y CI (formato, lints y tests).
- `nes-core`: `Emulator` sobre `tetanes-core` (carga de ROM, frame, audio, mandos, reset, save states, RAM con batería).
- `nes-core`: ROM de demostración propia (`demo_rom`) y tests headless de extremo a extremo.
- `nes-app`: ventana `eframe` con la imagen del juego, escalado entero y ritmo de emulación independiente de la frecuencia del monitor.
- `nes-app`: carga de ROMs por menú (`Ctrl+O`), arrastrar y soltar, línea de comandos y lista de recientes; ventana de error para archivos no válidos.
- `nes-app`: configuración persistente (recientes y escalado).
- `nes-app`: controles por teclado para dos jugadores y por mando (`gilrs`); ventana **Ayuda → Controles**.
- `nes-app`: salida de sonido con `cpal`, volumen y silencio (`M`), y control dinámico de velocidad para mantener el audio sincronizado.
- `nes-app`: pausa, reinicio, avance rápido ×4, save states en 4 ranuras y guardado automático de la RAM con batería.
- `nes-app`: ventana de ajustes con reasignación de teclas, proporción de televisor (8:7) y pantalla completa (`F11`).
- `nes-app`: versión web en WebAssembly con `trunk`: render WebGL, selector de archivos del navegador y guardado en `localStorage`.
- CI: job que compila y pasa clippy para `wasm32-unknown-unknown`.
- `nes-core`: ejemplo `dump_demo` para escribir la ROM de demostración en disco.
