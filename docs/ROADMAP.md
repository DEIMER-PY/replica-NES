# Roadmap

Cada fila es una rama `feature/*` que se integra en `develop` con su pull request.

## v0.1.0 — publicada el 2026-11-16

| Estado | Rama | Funcionalidad |
|---|---|---|
| ✅ | `chore/project-setup` | Workspace, documentación base y CI |
| ✅ | `feature/core-integration` | `Emulator` sobre `tetanes-core` con tests headless |
| ✅ | `feature/video-window` | Ventana con la imagen del juego, escalado y 60 FPS |
| ✅ | `feature/rom-loading` | Abrir ROM: menú, arrastrar y soltar, CLI, recientes |
| ✅ | `feature/input` | Teclado para dos jugadores y mandos |
| ✅ | `feature/audio` | Salida de sonido, volumen y silencio |
| ✅ | `feature/emulation-controls` | Pausa, reset, avance rápido, save states, guardado de batería |
| ✅ | `feature/settings` | Ajustes: teclas, escala, pantalla completa; persistencia |
| ✅ | `feature/web-wasm` | Versión web con `trunk` |

## v0.2.0 — publicada el 2026-11-16

| Estado | Rama | Funcionalidad |
|---|---|---|
| ✅ | `feature/gamepad-remapping` | Reasignación de los botones del mando por jugador |
| ✅ | `feature/performance` | Perfil `release` más rápido, frames omitidos en avance rápido, reposo en pausa, benchmark y workflow de ejecutables |
| ✅ | `docs/installation` | Guía de instalación, requisitos mínimos y fuentes de ROMs |
| ✅ | `feature/home-screen` | Pantalla de inicio con biblioteca de juegos |

## Pendiente de validar

Las versiones publicadas se verificaron con la ROM de demostración del proyecto. Falta probar con hardware y juegos reales:

- Juegos comerciales u homebrew con scroll, sprites y distintos mappers.
- Guardado de batería con un juego que lo use.
- Mandos físicos, incluida la reasignación de botones con un mando genérico.
- En escritorio: diálogos de archivo y de carpeta, arrastrar y soltar, pantalla completa y proporción 8:7.
- Ejecutable de Linux generado por el workflow `release`; macOS (sin compilar ni probar).
- Sonido en la versión web y navegadores distintos de Chromium.
- Linux y macOS (la CI compila y pasa los tests en Linux, pero la app no se ha ejecutado allí).

## Más adelante

- Visores de depuración: CPU, nametables, paletas, sprites.
- Rebobinado.
- Filtros de imagen (CRT, NTSC).
- Publicación de la versión web en GitHub Pages.
- Ejecutable para macOS en el workflow `release`.
- Carátulas o datos del juego (mapper, región) en la biblioteca.
- Core 6502/PPU propio como ejercicio, detrás de la API de `nes-core`.
