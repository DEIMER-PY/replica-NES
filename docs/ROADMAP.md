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

## v0.3.0 — publicada el 2026-11-18

| Estado | Rama | Funcionalidad |
|---|---|---|
| ✅ | `chore/release-hardening` | Ejecutable de Linux compilado en Ubuntu 22.04 (glibc 2.35) y prueba de arranque de 15 s de los ejecutables en la CI |
| ✅ | `feature/app-icon` | Icono propio en la ventana, el `.exe` y la web |
| ✅ | `feature/display-options` | Estirar, suavizado, líneas de barrido y giro |
| ✅ | `docs/showcase` | Capturas, pruebas con juegos en Windows y Linux, tecnologías y diagramas |
| ✅ | `release v0.3.0` | Publicación en `main` con el tag `v0.3.0` |

## Pendiente de validar

Contra (mapper UxROM) y Super Mario Bros. 3 (MMC3) se probaron en Windows y Linux: ver [PRUEBAS.md](PRUEBAS.md). Falta:

- Guardado de batería con un juego que lo use.
- Mandos físicos, incluida la reasignación de botones con un mando genérico.
- En escritorio: diálogos de archivo y de carpeta, arrastrar y soltar, pantalla completa y proporción 8:7.
- macOS (sin compilar ni probar).
- Sonido en la versión web y navegadores distintos de Chromium.

## Más adelante

- Visores de depuración: CPU, nametables, paletas, sprites.
- Rebobinado.
- Filtro NTSC (las líneas de barrido ya están).
- Publicación de la versión web en GitHub Pages.
- Ejecutable para macOS en el workflow `release`.
- Carátulas o datos del juego (mapper, región) en la biblioteca.
- Core 6502/PPU propio como ejercicio, detrás de la API de `nes-core`.
