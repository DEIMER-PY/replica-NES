# Arquitectura

## Principio

La emulación no se reimplementa: la aporta [`tetanes-core`](https://crates.io/crates/tetanes-core). El proyecto se divide en dos crates para que la app nunca dependa directamente de esa librería.

```
┌─────────────────────────────────────────────┐
│ nes-app  (binario: escritorio y web)        │
│  ventana, interfaz, audio, controles,       │
│  configuración, guardado en disco           │
└──────────────────────┬──────────────────────┘
                       │ API propia y pequeña
┌──────────────────────┴──────────────────────┐
│ nes-core  (librería)                        │
│  Emulator: cargar ROM, avanzar un frame,    │
│  imagen, audio, botones, save states        │
└──────────────────────┬──────────────────────┘
                       │
┌──────────────────────┴──────────────────────┐
│ tetanes-core  (dependencia externa)         │
│  CPU 6502, PPU, APU, mappers                │
└─────────────────────────────────────────────┘
```

Ventajas de esta separación:

- **Sustituible:** se puede cambiar de core, o escribir uno propio, sin tocar la app.
- **Testeable:** `nes-core` se prueba sin ventana ni audio (tests headless).
- **Portable:** `nes-core` no sabe nada de la plataforma; todo lo específico de escritorio o web vive en `nes-app`.

## Crates

| Crate | Tipo | Responsabilidad |
|---|---|---|
| `crates/nes-core` | librería | API de emulación independiente de la plataforma |
| `crates/nes-app` | binario | Aplicación `eframe`/`egui` para escritorio y WebAssembly |

Los módulos de cada crate se documentan aquí a medida que se implementan.
