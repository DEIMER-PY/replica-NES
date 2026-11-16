//! La aplicación: estado y lógica que unen el emulador con la ventana.
//!
//! La interfaz (menús y ventanas) está en [`crate::ui`]; ambos se comunican con
//! [`Action`]: un menú o un atajo producen una acción y [`NesApp::perform`] la ejecuta.

use std::path::{Path, PathBuf};

use eframe::egui::{self, Key, KeyboardShortcut, Modifiers};
use nes_core::{Button, Emulator, ResetKind};

use crate::{
    audio::AudioOutput,
    clock::{FrameClock, MAX_FRAMES_PER_UPDATE},
    config::Config,
    input::{Gamepads, PLAYERS},
    rom::RomFile,
    settings::SettingsWindow,
    storage::{SaveKind, SaveStore},
    video::Screen,
};

/// Nombre de la app, que encabeza el título de la ventana.
pub const APP_NAME: &str = "replica-NES";

/// Ranuras de save state disponibles.
pub const STATE_SLOTS: std::ops::RangeInclusive<u8> = 1..=4;

/// Tecla que acelera la emulación mientras se mantiene pulsada.
pub const FAST_FORWARD_KEY: Key = Key::F;
/// Multiplicador de velocidad del avance rápido.
const FAST_FORWARD_SPEED: f64 = 4.0;
/// Tope de frames por repintado durante el avance rápido.
const FAST_FORWARD_MAX_FRAMES: u32 = 8;

/// Corrección máxima de la velocidad de emulación para mantener estable la cola de
/// audio: ±0,5 %, imperceptible.
const MAX_RATE_CORRECTION: f64 = 0.005;

/// Segundos que permanece en pantalla un aviso.
const TOAST_SECONDS: f64 = 2.0;

/// Algo que el usuario puede pedir desde un menú o con un atajo de teclado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    OpenRom,
    LoadDemo,
    CloseRom,
    Quit,
    TogglePause,
    Reset(ResetKind),
    SaveState,
    LoadState,
    SelectSlot(u8),
    ToggleMute,
    ToggleFullscreen,
    OpenSettings,
}

impl Action {
    /// Acciones con atajo de teclado.
    ///
    /// El orden importa: un atajo con más modificadores va antes que otro con la misma
    /// tecla (`Ctrl+Shift+R` antes que `Ctrl+R`), porque se consume el primero que encaja.
    pub const SHORTCUTS: [(Self, KeyboardShortcut); 13] = [
        (Self::OpenRom, shortcut(Modifiers::COMMAND, Key::O)),
        (
            Self::Reset(ResetKind::Hard),
            shortcut(Modifiers::COMMAND.plus(Modifiers::SHIFT), Key::R),
        ),
        (
            Self::Reset(ResetKind::Soft),
            shortcut(Modifiers::COMMAND, Key::R),
        ),
        (Self::TogglePause, shortcut(Modifiers::NONE, Key::P)),
        (Self::SaveState, shortcut(Modifiers::NONE, Key::F5)),
        (Self::LoadState, shortcut(Modifiers::NONE, Key::F8)),
        (Self::ToggleMute, shortcut(Modifiers::NONE, Key::M)),
        (Self::ToggleFullscreen, shortcut(Modifiers::NONE, Key::F11)),
        (Self::OpenSettings, shortcut(Modifiers::COMMAND, Key::Comma)),
        (Self::SelectSlot(1), shortcut(Modifiers::NONE, Key::Num1)),
        (Self::SelectSlot(2), shortcut(Modifiers::NONE, Key::Num2)),
        (Self::SelectSlot(3), shortcut(Modifiers::NONE, Key::Num3)),
        (Self::SelectSlot(4), shortcut(Modifiers::NONE, Key::Num4)),
    ];

    /// Si `key` está reservada por la app y no puede asignarse a un botón del mando:
    /// los atajos de una sola tecla, el avance rápido y `Escape`.
    pub fn is_reserved_key(key: Key) -> bool {
        key == FAST_FORWARD_KEY
            || key == Key::Escape
            || Self::SHORTCUTS
                .iter()
                .any(|(_, shortcut)| shortcut.modifiers.is_none() && shortcut.logical_key == key)
    }

    /// Atajo de teclado de la acción, si tiene.
    pub fn shortcut(self) -> Option<KeyboardShortcut> {
        Self::SHORTCUTS
            .iter()
            .find(|(action, _)| *action == self)
            .map(|(_, shortcut)| *shortcut)
    }
}

const fn shortcut(modifiers: Modifiers, key: Key) -> KeyboardShortcut {
    KeyboardShortcut::new(modifiers, key)
}

/// Estado de la aplicación.
pub struct NesApp {
    pub(crate) emulator: Emulator,
    pub(crate) screen: Screen,
    clock: FrameClock,
    pub(crate) config: Config,
    pub(crate) gamepads: Gamepads,
    /// Salida de sonido; `None` si no se pudo abrir (la app sigue, en silencio).
    audio: Option<AudioOutput>,
    /// Motivo por el que no hay sonido, para mostrarlo en el menú Audio.
    pub(crate) audio_error: Option<String>,
    /// Si todavía no se ha intentado abrir la salida de sonido.
    audio_pending: bool,
    /// ROMs que el navegador termina de leer en segundo plano.
    #[cfg(target_arch = "wasm32")]
    rom_inbox: crate::web::RomInbox,
    saves: SaveStore,
    /// Identificador de la ROM cargada, con el que se nombran sus archivos guardados.
    rom_id: Option<String>,
    /// Última RAM con batería escrita en disco, para no reescribirla si no cambió.
    battery_on_disk: Option<Vec<u8>>,
    pub(crate) paused: bool,
    /// Si en el último repintado se estaba en avance rápido.
    pub(crate) fast_forward: bool,
    /// Ranura de save state seleccionada.
    pub(crate) state_slot: u8,
    /// Aviso breve y el instante en que deja de mostrarse.
    toast: Option<(String, f64)>,
    /// Estado de la ventana de ajustes.
    pub(crate) settings: SettingsWindow,
    /// Mensaje de error pendiente de mostrar al usuario.
    pub(crate) error: Option<String>,
}

impl NesApp {
    /// Crea la app. Carga `initial_rom` si se indica; si no, la ROM de demostración.
    pub fn new(cc: &eframe::CreationContext<'_>, initial_rom: Option<PathBuf>) -> Self {
        let mut app = Self {
            emulator: Emulator::new(),
            screen: Screen::default(),
            clock: FrameClock::default(),
            config: Config::load(cc.storage),
            gamepads: Gamepads::default(),
            audio: None,
            audio_error: None,
            audio_pending: true,
            #[cfg(target_arch = "wasm32")]
            rom_inbox: crate::web::RomInbox::default(),
            saves: SaveStore::for_app(APP_NAME),
            rom_id: None,
            battery_on_disk: None,
            paused: false,
            fast_forward: false,
            state_slot: *STATE_SLOTS.start(),
            toast: None,
            settings: SettingsWindow::default(),
            error: None,
        };
        // En escritorio el sonido se abre al arrancar. En la web hay que esperar a la
        // primera pulsación del usuario: los navegadores bloquean el audio hasta entonces.
        #[cfg(not(target_arch = "wasm32"))]
        app.open_audio();
        match initial_rom {
            Some(path) => app.open_path(&cc.egui_ctx, &path),
            None => app.perform(&cc.egui_ctx, Action::LoadDemo),
        }
        app
    }

    /// Ejecuta una acción pedida por un menú o un atajo.
    pub fn perform(&mut self, ctx: &egui::Context, action: Action) {
        match action {
            Action::OpenRom => self.open_dialog(ctx),
            Action::LoadDemo => {
                let demo = RomFile::from_bytes("demo.nes", nes_core::demo_rom());
                self.load_rom(ctx, &demo);
            }
            Action::CloseRom => self.close_rom(ctx),
            Action::Quit => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
            Action::ToggleMute => self.config.muted = !self.config.muted,
            Action::ToggleFullscreen => {
                let fullscreen = ctx.input(|input| input.viewport().fullscreen.unwrap_or(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fullscreen));
            }
            Action::OpenSettings => self.settings.open = true,
            Action::SelectSlot(slot) => {
                self.state_slot = slot;
                self.notify(ctx, format!("Ranura {slot}"));
            }
            // Las siguientes solo tienen sentido con un juego cargado.
            _ if !self.has_rom() => {}
            Action::TogglePause => self.paused = !self.paused,
            Action::Reset(kind) => {
                self.emulator.reset(kind);
                self.paused = false;
                self.clear_audio();
                self.notify(
                    ctx,
                    match kind {
                        ResetKind::Soft => "Reinicio",
                        ResetKind::Hard => "Apagado y encendido",
                    },
                );
            }
            Action::SaveState => self.save_state(ctx),
            Action::LoadState => self.load_state(ctx),
        }
    }

    /// Abre la salida de sonido y ajusta el emulador a su frecuencia de muestreo.
    fn open_audio(&mut self) {
        self.audio_pending = false;
        match AudioOutput::open() {
            Ok(audio) => {
                self.emulator.set_sample_rate(audio.sample_rate() as f32);
                self.audio = Some(audio);
                self.audio_error = None;
            }
            Err(err) => self.audio_error = Some(err),
        }
    }

    /// En la web, abre el sonido en cuanto el usuario pulsa una tecla o hace clic.
    #[cfg(target_arch = "wasm32")]
    fn open_audio_on_user_gesture(&mut self, ctx: &egui::Context) {
        if !self.audio_pending {
            return;
        }
        let gesture = ctx.input(|input| {
            input.pointer.any_pressed()
                || input
                    .events
                    .iter()
                    .any(|event| matches!(event, egui::Event::Key { pressed: true, .. }))
        });
        if gesture {
            self.open_audio();
        }
    }

    /// En la web, carga las ROMs que el navegador ha terminado de leer.
    #[cfg(target_arch = "wasm32")]
    fn receive_roms(&mut self, ctx: &egui::Context) {
        while let Some(result) = self.rom_inbox.poll() {
            match result {
                Ok(rom) => self.load_rom(ctx, &rom),
                Err(err) => self.error = Some(err),
            }
        }
    }

    /// Si hay un juego cargado.
    pub fn has_rom(&self) -> bool {
        self.emulator.rom().is_some()
    }

    /// Carga una ROM en el emulador, restaura su partida y la anota en recientes.
    fn load_rom(&mut self, ctx: &egui::Context, rom: &RomFile) {
        self.flush_battery();
        match self.emulator.load_rom(&rom.name, &rom.data) {
            Ok(info) => {
                let title = format!("{APP_NAME} — {}", info.name);
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
                self.rom_id = Some(rom.id.clone());
                self.error = None;
                self.paused = false;
                self.screen.clear();
                self.clear_audio();
                self.restore_battery();
                if let Some(path) = &rom.path {
                    self.config.recent_roms.push(path);
                }
            }
            Err(err) => {
                // El emulador queda sin cartucho tras un fallo de carga.
                self.close_rom(ctx);
                self.error = Some(format!("{}: {err}", rom.name));
            }
        }
    }

    /// Lee una ROM del disco y la carga.
    pub fn open_path(&mut self, ctx: &egui::Context, path: &Path) {
        match RomFile::read(path) {
            Ok(rom) => self.load_rom(ctx, &rom),
            Err(err) => {
                self.config.recent_roms.remove(path);
                self.error = Some(err);
            }
        }
    }

    /// Muestra el diálogo del sistema para elegir una ROM.
    #[cfg(not(target_arch = "wasm32"))]
    fn open_dialog(&mut self, ctx: &egui::Context) {
        let picked = rfd::FileDialog::new()
            .set_title("Abrir ROM")
            .add_filter("ROM de NES", &["nes"])
            .add_filter("Todos los archivos", &["*"])
            .pick_file();
        if let Some(path) = picked {
            self.open_path(ctx, &path);
        }
    }

    /// Abre el selector de archivos del navegador; la ROM llega después por el buzón.
    #[cfg(target_arch = "wasm32")]
    fn open_dialog(&mut self, _ctx: &egui::Context) {
        self.rom_inbox.pick_file();
    }

    /// Guarda la partida, extrae el cartucho y deja la pantalla vacía.
    fn close_rom(&mut self, ctx: &egui::Context) {
        self.flush_battery();
        self.emulator.unload_rom();
        self.rom_id = None;
        self.battery_on_disk = None;
        self.paused = false;
        self.screen.clear();
        self.clear_audio();
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(APP_NAME.to_owned()));
    }

    /// Restaura desde disco la partida guardada de la ROM recién cargada.
    fn restore_battery(&mut self) {
        self.battery_on_disk = None;
        let Some(rom_id) = &self.rom_id else {
            return;
        };
        if self.emulator.battery_ram().is_none() {
            return;
        }
        match self.saves.read(SaveKind::Battery, rom_id) {
            Ok(Some(data)) => {
                self.emulator.set_battery_ram(&data);
                self.battery_on_disk = Some(data);
            }
            Ok(None) => {}
            Err(err) => self.error = Some(err),
        }
    }

    /// Escribe en disco la partida guardada del juego, si tiene y ha cambiado.
    fn flush_battery(&mut self) {
        let Some(rom_id) = &self.rom_id else {
            return;
        };
        let Some(data) = self.emulator.battery_ram() else {
            return;
        };
        if self.battery_on_disk.as_ref() == Some(&data) {
            return;
        }
        match self.saves.write(SaveKind::Battery, rom_id, &data) {
            Ok(()) => self.battery_on_disk = Some(data),
            Err(err) => self.error = Some(err),
        }
    }

    /// Guarda el estado de la consola en la ranura seleccionada.
    fn save_state(&mut self, ctx: &egui::Context) {
        let Some(rom_id) = self.rom_id.clone() else {
            return;
        };
        let slot = self.state_slot;
        let result = self
            .emulator
            .save_state()
            .map_err(|err| err.to_string())
            .and_then(|data| self.saves.write(SaveKind::State(slot), &rom_id, &data));
        match result {
            Ok(()) => self.notify(ctx, format!("Estado guardado en la ranura {slot}")),
            Err(err) => self.error = Some(err),
        }
    }

    /// Restaura el estado de la ranura seleccionada.
    fn load_state(&mut self, ctx: &egui::Context) {
        let Some(rom_id) = self.rom_id.clone() else {
            return;
        };
        let slot = self.state_slot;
        match self.saves.read(SaveKind::State(slot), &rom_id) {
            Ok(Some(data)) => match self.emulator.load_state(&data) {
                Ok(()) => {
                    self.clear_audio();
                    // El estado no incluye la imagen: se emula un frame para tenerla,
                    // de modo que también se vea al cargar con la consola en pausa.
                    if let Err(err) = self.emulator.run_frame() {
                        self.error = Some(err.to_string());
                        return;
                    }
                    self.screen.upload(ctx, self.emulator.frame());
                    self.notify(ctx, format!("Estado cargado de la ranura {slot}"));
                }
                Err(err) => self.error = Some(err.to_string()),
            },
            Ok(None) => self.notify(ctx, format!("La ranura {slot} está vacía")),
            Err(err) => self.error = Some(err),
        }
    }

    /// Muestra un aviso breve sobre la imagen.
    fn notify(&mut self, ctx: &egui::Context, message: impl Into<String>) {
        let until = ctx.input(|input| input.time) + TOAST_SECONDS;
        self.toast = Some((message.into(), until));
    }

    /// Aviso vigente, si lo hay.
    pub fn toast(&mut self, ctx: &egui::Context) -> Option<&str> {
        let now = ctx.input(|input| input.time);
        if self.toast.as_ref().is_some_and(|(_, until)| now >= *until) {
            self.toast = None;
        }
        self.toast.as_ref().map(|(message, _)| message.as_str())
    }

    /// Descarta el sonido pendiente (cambio de ROM, reinicio, carga de estado).
    fn clear_audio(&self) {
        if let Some(audio) = &self.audio {
            audio.clear();
        }
    }

    /// Ejecuta los atajos de teclado pulsados.
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        // Al reasignar una tecla, la pulsación es para los ajustes, no un atajo.
        if self.settings.is_capturing() {
            return;
        }
        // Mientras se escribe en la interfaz, las teclas sueltas no son atajos.
        let typing = ctx.egui_wants_keyboard_input();
        for (action, shortcut) in Action::SHORTCUTS {
            if typing && shortcut.modifiers.is_none() {
                continue;
            }
            if ctx.input_mut(|input| input.consume_shortcut(&shortcut)) {
                self.perform(ctx, action);
            }
        }
    }

    /// Carga el primer archivo soltado sobre la ventana, si lo hay.
    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|input| input.raw.dropped_files.first().cloned());
        let Some(file) = dropped else {
            return;
        };
        #[cfg(not(target_arch = "wasm32"))]
        self.open_path(ctx, file.path());
        // En la web el archivo no tiene ruta: se lee de forma asíncrona.
        #[cfg(target_arch = "wasm32")]
        {
            let _ = ctx;
            self.rom_inbox.read_dropped(file);
        }
    }

    /// Pasa al emulador el estado de teclado y mandos.
    fn apply_input(&mut self, ctx: &egui::Context) {
        let keyboard = if self.settings.is_capturing() {
            // La tecla que se está asignando no debe llegar al juego.
            Default::default()
        } else {
            self.config.key_bindings.read(ctx)
        };
        let gamepads = self.gamepads.read();
        for (index, player) in PLAYERS.into_iter().enumerate() {
            let pad = keyboard[index].merge(gamepads[index]);
            for button in Button::ALL {
                self.emulator.set_button(player, button, pad.get(button));
            }
        }
    }

    /// Emula los frames que tocan desde el repintado anterior.
    fn run_emulation(&mut self, ctx: &egui::Context) {
        self.fast_forward = false;
        if !self.has_rom() {
            return;
        }
        if let Some(audio) = &self.audio {
            audio.set_volume(f32::from(self.config.volume) / 100.0, self.config.muted);
        }
        if self.paused {
            self.clock.reset();
            return;
        }

        self.fast_forward = !ctx.egui_wants_keyboard_input()
            && !self.settings.is_capturing()
            && ctx.input(|input| input.key_down(FAST_FORWARD_KEY));

        let now = ctx.input(|input| input.time);
        let mut frame_rate = self.emulator.region().frame_rate();
        let mut max_frames = MAX_FRAMES_PER_UPDATE;
        if self.fast_forward {
            frame_rate *= FAST_FORWARD_SPEED;
            max_frames = FAST_FORWARD_MAX_FRAMES;
        } else if let Some(audio) = &self.audio {
            // El reloj de la tarjeta de sonido y el de la pantalla nunca coinciden del
            // todo. Con la cola de audio baja se emula un poco más rápido y con la cola
            // alta un poco más lento, para que ni se vacíe ni se acumule retraso.
            let error = f64::from(audio.fill_error()).clamp(-1.0, 1.0);
            frame_rate *= 1.0 - error * MAX_RATE_CORRECTION;
        }
        let frames = self.clock.frames_due(now, frame_rate, max_frames);
        self.apply_input(ctx);

        for _ in 0..frames {
            if let Err(err) = self.emulator.run_frame() {
                self.close_rom(ctx);
                self.error = Some(err.to_string());
                return;
            }
            // En avance rápido el sonido iría acelerado y desbordaría la cola: se omite.
            if !self.fast_forward
                && let Some(audio) = &self.audio
            {
                audio.push(self.emulator.audio_samples());
            }
        }
        if frames > 0 {
            self.screen.upload(ctx, self.emulator.frame());
        }
    }
}

impl eframe::App for NesApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(target_arch = "wasm32")]
        {
            self.open_audio_on_user_gesture(ctx);
            self.receive_roms(ctx);
        }
        self.handle_shortcuts(ctx);
        self.handle_dropped_files(ctx);
        self.run_emulation(ctx);
        // La emulación es continua: se pide repintar siempre, sin esperar a eventos.
        ctx.request_repaint();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.show(ui);
    }

    /// `eframe` lo llama cada 30 segundos y al cerrar la app.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.flush_battery();
        self.config.save(storage);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shortcut_is_unique() {
        for (i, (_, a)) in Action::SHORTCUTS.iter().enumerate() {
            for (_, b) in &Action::SHORTCUTS[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }

    #[test]
    fn specific_shortcuts_come_before_general_ones() {
        // Ctrl+Shift+R debe evaluarse antes que Ctrl+R, o nunca se alcanzaría.
        let position = |target: Action| {
            Action::SHORTCUTS
                .iter()
                .position(|(action, _)| *action == target)
                .unwrap()
        };
        assert!(
            position(Action::Reset(ResetKind::Hard)) < position(Action::Reset(ResetKind::Soft))
        );
    }

    #[test]
    fn slot_shortcuts_cover_all_slots() {
        for slot in STATE_SLOTS {
            assert!(
                Action::SelectSlot(slot).shortcut().is_some(),
                "ranura {slot}"
            );
        }
        assert!(Action::Quit.shortcut().is_none());
    }

    #[test]
    fn shortcuts_do_not_use_default_game_keys() {
        use crate::input::KeyBindings;

        let bindings = KeyBindings::default();
        let game_keys: Vec<Key> = PLAYERS
            .iter()
            .flat_map(|&player| Button::ALL.map(|button| bindings.map(player).key(button)))
            .collect();
        for (action, shortcut) in Action::SHORTCUTS {
            if shortcut.modifiers.is_none() {
                assert!(
                    !game_keys.contains(&shortcut.logical_key),
                    "{action:?} choca con una tecla de juego"
                );
            }
        }
        assert!(!game_keys.contains(&FAST_FORWARD_KEY));
    }
}
