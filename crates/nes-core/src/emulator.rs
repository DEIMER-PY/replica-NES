//! [`Emulator`]: la consola emulada.

use tetanes_core::{
    common::{NesRegion, ResetKind as CoreResetKind},
    control_deck::{Config, ControlDeck},
    input::{JoypadBtn, Player as CorePlayer},
    video::VideoFilter,
};

use crate::{DEFAULT_SAMPLE_RATE, FRAME_BYTES};

/// Errores de la emulación.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// La ROM no es válida o usa un mapper no soportado.
    #[error("no se pudo cargar la ROM: {0}")]
    LoadRom(String),
    /// Se pidió una operación que necesita una ROM cargada.
    #[error("no hay ninguna ROM cargada")]
    NoRom,
    /// La CPU se detuvo (opcode inválido) o falló la emulación.
    #[error("error de emulación: {0}")]
    Emulation(String),
    /// No se pudo guardar o restaurar el estado.
    #[error("error en el estado guardado: {0}")]
    SaveState(String),
}

/// Botones del mando de la NES.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Button {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}

impl Button {
    /// Todos los botones, en el orden en que se muestran en la interfaz.
    pub const ALL: [Self; 8] = [
        Self::Up,
        Self::Down,
        Self::Left,
        Self::Right,
        Self::A,
        Self::B,
        Self::Select,
        Self::Start,
    ];
}

/// Puerto de mando.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    One,
    Two,
}

/// Región de la consola, que determina la velocidad de la emulación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Region {
    /// América y Japón: ~60 frames por segundo.
    Ntsc,
    /// Europa: ~50 frames por segundo.
    Pal,
    /// Clones "Dendy": ~50 frames por segundo.
    Dendy,
}

impl Region {
    /// Frames por segundo de la región.
    #[must_use]
    pub const fn frame_rate(self) -> f64 {
        match self {
            Self::Ntsc => 60.0988,
            Self::Pal | Self::Dendy => 50.007,
        }
    }
}

/// Tipo de reinicio de la consola.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResetKind {
    /// Botón RESET: conserva la RAM.
    Soft,
    /// Apagar y encender: borra la RAM.
    Hard,
}

/// Datos de la ROM cargada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RomInfo {
    /// Nombre con el que se cargó la ROM.
    pub name: String,
    /// Si el cartucho guarda partida con batería (SRAM).
    pub battery_backed: bool,
    /// Región detectada.
    pub region: Region,
}

/// Una consola NES emulada.
///
/// El ciclo de uso es: cargar una ROM con [`Emulator::load_rom`], y por cada frame
/// de pantalla fijar los botones con [`Emulator::set_button`], llamar a
/// [`Emulator::run_frame`] y recoger la imagen ([`Emulator::frame`]) y el audio
/// ([`Emulator::audio_samples`]).
///
/// No accede al disco: las partidas y estados se entregan como bytes para que la
/// app decida dónde guardarlos (también en la web, donde no hay sistema de archivos).
#[derive(Debug)]
pub struct Emulator {
    deck: ControlDeck,
    rom: Option<RomInfo>,
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}

impl Emulator {
    /// Crea una consola sin cartucho, con audio a [`DEFAULT_SAMPLE_RATE`].
    #[must_use]
    pub fn new() -> Self {
        let config = Config::default()
            .with_filter(VideoFilter::Pixellate)
            .with_region(NesRegion::Auto)
            .with_sram_dir(None);
        let mut deck = ControlDeck::with_config(config);
        deck.set_sample_rate(DEFAULT_SAMPLE_RATE);
        Self { deck, rom: None }
    }

    /// Carga una ROM en formato iNES / NES 2.0 desde memoria y enciende la consola.
    ///
    /// # Errors
    ///
    /// Si la cabecera no es válida o el mapper no está soportado.
    pub fn load_rom(&mut self, name: &str, data: &[u8]) -> Result<&RomInfo, Error> {
        self.rom = None;
        let mut reader = data;
        let loaded = self
            .deck
            .load_rom(name, &mut reader)
            .map_err(|err| Error::LoadRom(err.to_string()))?;
        let info = RomInfo {
            name: loaded.name,
            battery_backed: loaded.battery_backed,
            region: match self.deck.region() {
                NesRegion::Pal => Region::Pal,
                NesRegion::Dendy => Region::Dendy,
                NesRegion::Ntsc | NesRegion::Auto => Region::Ntsc,
            },
        };
        Ok(self.rom.insert(info))
    }

    /// Extrae el cartucho.
    pub fn unload_rom(&mut self) {
        // El único fallo posible es escribir la SRAM en disco, y está desactivado.
        let _ = self.deck.unload_rom();
        self.rom = None;
    }

    /// Datos de la ROM cargada, si la hay.
    #[must_use]
    pub const fn rom(&self) -> Option<&RomInfo> {
        self.rom.as_ref()
    }

    /// Región en uso: la de la ROM cargada o NTSC si no hay ninguna.
    #[must_use]
    pub fn region(&self) -> Region {
        self.rom.as_ref().map_or(Region::Ntsc, |rom| rom.region)
    }

    /// Emula un frame completo de vídeo (y el audio correspondiente).
    ///
    /// # Errors
    ///
    /// Si no hay ROM cargada o la CPU encuentra un opcode inválido.
    pub fn run_frame(&mut self) -> Result<(), Error> {
        if self.rom.is_none() {
            return Err(Error::NoRom);
        }
        self.deck
            .clock_frame()
            .map(|_| ())
            .map_err(|err| Error::Emulation(err.to_string()))
    }

    /// Imagen del último frame: [`FRAME_BYTES`] bytes RGBA, 256×240, fila a fila.
    #[must_use]
    pub fn frame(&mut self) -> &[u8; FRAME_BYTES] {
        self.deck.frame_buffer()
    }

    /// Muestras de audio mono (`f32`, de -1 a 1) generadas por el último
    /// [`Emulator::run_frame`], a la frecuencia configurada.
    #[must_use]
    pub fn audio_samples(&self) -> &[f32] {
        self.deck.audio_samples()
    }

    /// Cambia la frecuencia de muestreo del audio, en Hz.
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.deck.set_sample_rate(sample_rate);
    }

    /// Pulsa o suelta un botón de un mando.
    pub fn set_button(&mut self, player: Player, button: Button, pressed: bool) {
        let player = match player {
            Player::One => CorePlayer::One,
            Player::Two => CorePlayer::Two,
        };
        let button = match button {
            Button::A => JoypadBtn::A,
            Button::B => JoypadBtn::B,
            Button::Select => JoypadBtn::Select,
            Button::Start => JoypadBtn::Start,
            Button::Up => JoypadBtn::Up,
            Button::Down => JoypadBtn::Down,
            Button::Left => JoypadBtn::Left,
            Button::Right => JoypadBtn::Right,
        };
        self.deck.joypad_mut(player).set_button(button, pressed);
    }

    /// Reinicia la consola.
    pub fn reset(&mut self, kind: ResetKind) {
        self.deck.reset(match kind {
            ResetKind::Soft => CoreResetKind::Soft,
            ResetKind::Hard => CoreResetKind::Hard,
        });
    }

    /// Serializa el estado completo de la consola (save state).
    ///
    /// # Errors
    ///
    /// Si no hay ROM cargada o falla la serialización.
    pub fn save_state(&mut self) -> Result<Vec<u8>, Error> {
        if self.rom.is_none() {
            return Err(Error::NoRom);
        }
        let mut data = Vec::new();
        self.deck
            .save_state(&mut data)
            .map_err(|err| Error::SaveState(err.to_string()))?;
        Ok(data)
    }

    /// Restaura un estado creado con [`Emulator::save_state`] para la misma ROM.
    ///
    /// # Errors
    ///
    /// Si no hay ROM cargada, los datos están corruptos o son de otra ROM.
    pub fn load_state(&mut self, data: &[u8]) -> Result<(), Error> {
        if self.rom.is_none() {
            return Err(Error::NoRom);
        }
        self.deck
            .load_state(data)
            .map_err(|err| Error::SaveState(err.to_string()))
    }

    /// RAM con batería del cartucho (la partida guardada), o `None` si la ROM
    /// cargada no la tiene.
    #[must_use]
    pub fn battery_ram(&mut self) -> Option<Vec<u8>> {
        if self.rom.as_ref()?.battery_backed {
            Some(self.deck.sram().to_vec())
        } else {
            None
        }
    }

    /// Restaura la RAM con batería del cartucho. Se llama justo después de cargar la ROM.
    pub fn set_battery_ram(&mut self, data: &[u8]) {
        if self.rom.as_ref().is_some_and(|rom| rom.battery_backed) {
            self.deck.set_sram(data);
        }
    }
}
