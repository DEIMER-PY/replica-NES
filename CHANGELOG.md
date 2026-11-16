# Changelog

Todos los cambios relevantes del proyecto se registran aquí.
El formato sigue [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) y el proyecto usa [versionado semántico](https://semver.org/lang/es/).

## [Sin publicar]

### Añadido

- Estructura del workspace con los crates `nes-core` y `nes-app`.
- Documentación base, guía de desarrollo y CI (formato, lints y tests).
- `nes-core`: `Emulator` sobre `tetanes-core` (carga de ROM, frame, audio, mandos, reset, save states, RAM con batería).
- `nes-core`: ROM de demostración propia (`demo_rom`) y tests headless de extremo a extremo.
