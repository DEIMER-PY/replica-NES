# replica-NES

Emulador de la consola NES escrito en Rust: app nativa de escritorio con interfaz gráfica y, más adelante, versión web (WebAssembly).

> Proyecto en desarrollo. El estado de cada funcionalidad está en [docs/ROADMAP.md](docs/ROADMAP.md).

## Qué es

`replica-NES` carga una ROM en formato iNES (`.nes`) y la ejecuta emulando el hardware de la consola: CPU 6502, PPU (vídeo), APU (audio) y los mappers de los cartuchos.

La emulación la aporta [`tetanes-core`](https://crates.io/crates/tetanes-core), un core probado y preciso. Este proyecto construye el producto alrededor: ventana, interfaz, audio, controles, guardado y versión web.

## Tecnologías

| Pieza | Herramienta |
|---|---|
| Emulación | `tetanes-core` |
| Ventana e interfaz | `eframe` / `egui` (render con `wgpu`) |
| Audio | `cpal` |
| Mandos | `gilrs` |
| Diálogo de archivos | `rfd` |
| Web | `trunk` + `wasm32-unknown-unknown` |

## Inicio rápido

Requisitos: [Rust](https://rustup.rs) estable (ver `rust-toolchain.toml`) y, en Windows, las C++ Build Tools de Visual Studio.

```sh
git clone https://github.com/DEIMER-PY/replica-NES.git
cd replica-NES
cargo run -p nes-app --release
```

## Documentación

| Documento | Contenido |
|---|---|
| [docs/ARQUITECTURA.md](docs/ARQUITECTURA.md) | Crates, responsabilidades y flujo de un frame |
| [docs/HARDWARE-NES.md](docs/HARDWARE-NES.md) | Cómo funciona la NES: CPU, PPU, APU, mappers, formato iNES |
| [docs/GUIA-DESARROLLO.md](docs/GUIA-DESARROLLO.md) | Entorno, flujo de ramas y convención de commits |
| [docs/USO.md](docs/USO.md) | Cargar ROMs, controles y atajos |
| [docs/ROADMAP.md](docs/ROADMAP.md) | Funcionalidades hechas y pendientes |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Cómo contribuir |
| [CHANGELOG.md](CHANGELOG.md) | Historial de cambios |

## ROMs

Este repositorio **no incluye ni distribuye ROMs**. Usa volcados de tus propios cartuchos o juegos homebrew de libre distribución. Los archivos `.nes` están excluidos en `.gitignore`.

## Licencia

[MIT](LICENSE). `tetanes-core` se distribuye bajo MIT o Apache-2.0.
