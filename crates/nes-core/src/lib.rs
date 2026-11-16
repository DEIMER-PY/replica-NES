//! API de emulación de NES de replica-NES.
//!
//! Este crate aísla al resto del proyecto del core de emulación concreto
//! ([`tetanes-core`](https://crates.io/crates/tetanes-core)). La app solo conoce
//! [`Emulator`] y los tipos de este crate, de modo que el core se puede sustituir
//! sin tocarla.
//!
//! ```
//! use nes_core::{Button, Emulator, Player};
//!
//! let mut nes = Emulator::new();
//! nes.load_rom("demo", &nes_core::demo_rom())?;
//! nes.set_button(Player::One, Button::A, true);
//! nes.run_frame()?;
//!
//! assert_eq!(nes.frame().len(), nes_core::FRAME_BYTES);
//! # Ok::<(), nes_core::Error>(())
//! ```

mod demo;
mod emulator;

pub use demo::demo_rom;
pub use emulator::{Button, Emulator, Error, Player, Region, ResetKind, RomInfo};

/// Ancho de la imagen de la NES en píxeles.
pub const SCREEN_WIDTH: usize = 256;
/// Alto de la imagen de la NES en píxeles.
pub const SCREEN_HEIGHT: usize = 240;
/// Tamaño en bytes de un frame RGBA (4 bytes por píxel).
pub const FRAME_BYTES: usize = SCREEN_WIDTH * SCREEN_HEIGHT * 4;
/// Frecuencia de muestreo de audio por defecto, en Hz.
pub const DEFAULT_SAMPLE_RATE: f32 = 48_000.0;
