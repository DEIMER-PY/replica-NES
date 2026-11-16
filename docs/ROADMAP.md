# Roadmap

Cada fila es una rama `feature/*` que se integra en `develop` con su pull request.

## v0.1.0

| Estado | Rama | Funcionalidad |
|---|---|---|
| ✅ | `chore/project-setup` | Workspace, documentación base y CI |
| ✅ | `feature/core-integration` | `Emulator` sobre `tetanes-core` con tests headless |
| ⬜ | `feature/video-window` | Ventana con la imagen del juego, escalado y 60 FPS |
| ⬜ | `feature/rom-loading` | Abrir ROM: menú, arrastrar y soltar, CLI, recientes |
| ⬜ | `feature/input` | Teclado para dos jugadores y mandos |
| ⬜ | `feature/audio` | Salida de sonido, volumen y silencio |
| ⬜ | `feature/emulation-controls` | Pausa, reset, avance rápido, save states, guardado de batería |
| ⬜ | `feature/settings` | Ajustes: teclas, escala, pantalla completa; persistencia |
| ⬜ | `feature/web-wasm` | Versión web con `trunk` |

## Más adelante

- Visores de depuración: CPU, nametables, paletas, sprites.
- Rebobinado.
- Filtros de imagen (CRT, NTSC).
- Publicación de la versión web en GitHub Pages.
- Core 6502/PPU propio como ejercicio, detrás de la API de `nes-core`.
