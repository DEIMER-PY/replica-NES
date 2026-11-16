# Cómo funciona la NES

Resumen del hardware que emula el proyecto. Sirve para entender qué hace el core y cómo se conecta con la app. La referencia completa es la [NESdev Wiki](https://www.nesdev.org/wiki/).

## Vista general

```
              ┌──────────────┐
  mandos ───► │     CPU      │ ◄──► RAM 2 KB
              │ Ricoh 2A03   │
              │ (6502 + APU) │ ───► audio
              └──────┬───────┘
                     │ bus de CPU
              ┌──────┴───────┐        ┌──────────────┐
              │   Cartucho   │ ◄────► │     PPU      │ ───► vídeo 256×240
              │ PRG + CHR +  │  bus   │ Ricoh 2C02   │
              │   mapper     │ de PPU └──────────────┘
              └──────────────┘
```

## CPU (Ricoh 2A03)

- Es un MOS 6502 de 8 bits sin modo decimal, a ~1,79 MHz en NTSC.
- Registros: `A` (acumulador), `X`, `Y` (índices), `SP` (pila), `PC` (contador de programa, 16 bits) y `P` (flags).
- 56 instrucciones oficiales con 13 modos de direccionamiento (151 opcodes), más opcodes no oficiales que algunos juegos usan.
- Tres interrupciones: `RESET`, `NMI` (la lanza la PPU al empezar el vblank) e `IRQ` (APU y algunos mappers).

### Mapa de memoria de la CPU

| Rango | Contenido |
|---|---|
| `$0000–$07FF` | RAM interna (2 KB), reflejada hasta `$1FFF` |
| `$2000–$2007` | Registros de la PPU, reflejados hasta `$3FFF` |
| `$4000–$4017` | APU y mandos |
| `$4020–$5FFF` | Expansión del cartucho |
| `$6000–$7FFF` | RAM del cartucho (partidas guardadas con batería) |
| `$8000–$FFFF` | PRG-ROM: el código del juego |

## PPU (Ricoh 2C02)

Genera la imagen. Corre tres veces más rápido que la CPU: 3 ciclos de PPU por cada ciclo de CPU.

- Un frame NTSC son 262 líneas de 341 ciclos. Las líneas 0–239 son visibles, la 241 inicia el vblank (y dispara la NMI) y la 261 es la de pre-render.
- **Fondo:** una cuadrícula de 32×30 tiles de 8×8 píxeles (nametable), con scroll.
- **Sprites:** hasta 64 en memoria (OAM), máximo 8 por línea; de 8×8 u 8×16.
- **Color:** paleta fija de 64 colores; en pantalla hay 4 paletas de fondo y 4 de sprites, de 3 colores más transparente cada una.
- Los gráficos (tiles) viven en la CHR-ROM o CHR-RAM del cartucho.

Los juegos dependen de detalles finos de sincronización (por ejemplo el "sprite 0 hit" para dividir la pantalla), y por eso la precisión de la PPU es lo más difícil de un emulador.

## APU

Genera el sonido con cinco canales: dos de onda cuadrada, uno triangular, uno de ruido y uno de muestras (DMC). El emulador produce muestras de audio que la app envía a la tarjeta de sonido.

## Mandos

Cada mando tiene 8 botones (A, B, Select, Start, arriba, abajo, izquierda, derecha). La CPU los lee bit a bit por `$4016` (jugador 1) y `$4017` (jugador 2) tras escribir un pulso de "strobe".

## Cartuchos y mappers

La CPU solo ve 32 KB de programa y la PPU 8 KB de gráficos. Para superar ese límite, los cartuchos incluyen un chip (**mapper**) que cambia qué banco de memoria está visible en cada momento. Cada mapper es hardware distinto que hay que emular.

| Mapper | Nombre | Uso |
|---|---|---|
| 0 | NROM | Los juegos más simples, sin cambio de bancos |
| 1 | MMC1 | Muy común; cambio de bancos por registro serie |
| 2 | UxROM | Cambio de bancos de PRG |
| 3 | CNROM | Cambio de bancos de CHR |
| 4 | MMC3 | Muy común; añade contador de líneas con IRQ |

Con estos cinco se cubre la gran mayoría del catálogo. `tetanes-core` implementa bastantes más.

## Formato de ROM iNES

Un archivo `.nes` es el volcado del cartucho con una cabecera de 16 bytes:

| Bytes | Contenido |
|---|---|
| 0–3 | Firma `NES` seguida de `$1A` |
| 4 | Tamaño de PRG-ROM en bloques de 16 KB |
| 5 | Tamaño de CHR-ROM en bloques de 8 KB (0 = usa CHR-RAM) |
| 6 | Flags: mirroring, batería, trainer, nibble bajo del mapper |
| 7 | Flags: nibble alto del mapper, formato NES 2.0 |
| 8–15 | Extensiones (NES 2.0) o relleno |

Después van, en orden: el trainer opcional (512 bytes), la PRG-ROM y la CHR-ROM.

## Regiones

| | NTSC | PAL |
|---|---|---|
| Frames por segundo | 60,10 | 50,01 |
| Líneas por frame | 262 | 312 |
| Reloj de CPU | 1,79 MHz | 1,66 MHz |
