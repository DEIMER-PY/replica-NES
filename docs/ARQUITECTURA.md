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

## `nes-core`

| Módulo | Contenido |
|---|---|
| `emulator.rs` | `Emulator` y sus tipos: `Button`, `Player`, `Region`, `ResetKind`, `RomInfo`, `Error` |
| `demo.rs` | `demo_rom()`: ROM iNES propia generada en memoria, para tests y pantalla de prueba |
| `lib.rs` | Constantes de pantalla y audio (`SCREEN_WIDTH`, `SCREEN_HEIGHT`, `FRAME_BYTES`, `DEFAULT_SAMPLE_RATE`) |

### API de `Emulator`

| Método | Qué hace |
|---|---|
| `new()` | Consola sin cartucho, audio a 48 kHz |
| `load_rom(nombre, bytes)` | Carga una ROM iNES / NES 2.0 desde memoria y devuelve su `RomInfo` |
| `unload_rom()` | Extrae el cartucho |
| `run_frame()` | Emula un frame completo de vídeo y su audio |
| `frame()` | Imagen del último frame: 256×240 RGBA |
| `audio_samples()` | Muestras mono `f32` del último frame |
| `set_sample_rate(hz)` | Ajusta el audio a la frecuencia del dispositivo de salida |
| `set_button(jugador, botón, pulsado)` | Estado de un botón del mando |
| `reset(tipo)` | Reinicio suave (botón RESET) o duro (apagar y encender) |
| `save_state()` / `load_state(bytes)` | Estado completo de la consola como bytes |
| `battery_ram()` / `set_battery_ram(bytes)` | Partida guardada del cartucho (SRAM) |

### Decisiones

- **Sin acceso a disco.** ROMs, estados y partidas entran y salen como bytes. Así el mismo crate sirve en escritorio y en la web, y es la app quien decide dónde se guarda cada cosa.
- **Tipos propios.** `nes-core` no reexporta ningún tipo de `tetanes-core`; los errores se convierten a `nes_core::Error`.
- **Imagen sin filtro.** Se usa el filtro de píxel directo (un píxel RGBA por píxel de la PPU). El filtro NTSC queda como opción futura.

### Flujo de un frame

```
app                         nes-core                 tetanes-core
 │  set_button(...)  ───────►  joypad
 │  run_frame()      ───────►  clock_frame()  ─────►  CPU + PPU + APU
 │  frame()          ◄───────  256×240 RGBA
 │  audio_samples()  ◄───────  ~800 muestras (48 kHz / 60 fps)
 ▼
 textura en pantalla + cola de audio
```

### ROM de demostración

`demo_rom()` devuelve un programa 6502 de 96 bytes ensamblado a mano (mapper 0): pantalla azul y silencio con el botón A suelto; pantalla roja y un tono de ~440 Hz mientras se mantiene pulsado. Con ella los tests comprueban vídeo, audio, mandos, save states y reset sin depender de ninguna ROM externa.

## `nes-app`

Los módulos se documentan aquí a medida que se implementan.
