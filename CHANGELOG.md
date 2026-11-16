# Changelog

Todos los cambios relevantes del proyecto se registran aquí.
El formato sigue [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) y el proyecto usa [versionado semántico](https://semver.org/lang/es/).

## [Sin publicar]

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
