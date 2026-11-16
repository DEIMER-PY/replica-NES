//! La aplicación: une el emulador con la ventana.

use std::path::{Path, PathBuf};

use eframe::egui::{self, Key, KeyboardShortcut, Modifiers};
use nes_core::{Button, Emulator, Player};

use crate::{
    audio::AudioOutput,
    clock::FrameClock,
    config::Config,
    input::{Gamepads, PLAYERS},
    rom::RomFile,
    video::Screen,
};

/// Nombre de la app, que encabeza el título de la ventana.
pub const APP_NAME: &str = "replica-NES";

/// Atajo de "Abrir ROM".
const OPEN_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::COMMAND, Key::O);
/// Atajo de "Silenciar".
const MUTE_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::NONE, Key::M);

/// Corrección máxima de la velocidad de emulación para mantener estable la cola de
/// audio: ±0,5 %, imperceptible.
const MAX_RATE_CORRECTION: f64 = 0.005;

/// Estado de la aplicación.
pub struct NesApp {
    emulator: Emulator,
    screen: Screen,
    clock: FrameClock,
    config: Config,
    gamepads: Gamepads,
    /// Salida de sonido; `None` si no se pudo abrir (la app sigue, en silencio).
    audio: Option<AudioOutput>,
    /// Motivo por el que no hay sonido, para mostrarlo en el menú Audio.
    audio_error: Option<String>,
    /// Si está abierta la ventana de ayuda de controles.
    show_controls: bool,
    /// Mensaje de error pendiente de mostrar al usuario.
    error: Option<String>,
}

impl NesApp {
    /// Crea la app. Carga `initial_rom` si se indica; si no, la ROM de demostración.
    pub fn new(cc: &eframe::CreationContext<'_>, initial_rom: Option<PathBuf>) -> Self {
        let (audio, audio_error) = match AudioOutput::open() {
            Ok(audio) => (Some(audio), None),
            Err(err) => (None, Some(err)),
        };
        let mut emulator = Emulator::new();
        if let Some(audio) = &audio {
            emulator.set_sample_rate(audio.sample_rate() as f32);
        }
        let mut app = Self {
            emulator,
            screen: Screen::default(),
            clock: FrameClock::default(),
            config: Config::load(cc.storage),
            gamepads: Gamepads::default(),
            audio,
            audio_error,
            show_controls: false,
            error: None,
        };
        match initial_rom {
            Some(path) => app.open_path(&cc.egui_ctx, &path),
            None => app.load_demo(&cc.egui_ctx),
        }
        app
    }

    /// Carga una ROM en el emulador y la anota en recientes si viene del disco.
    fn load_rom(&mut self, ctx: &egui::Context, rom: &RomFile) {
        match self.emulator.load_rom(&rom.name, &rom.data) {
            Ok(info) => {
                let title = format!("{APP_NAME} — {}", info.name);
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
                self.error = None;
                self.screen.clear();
                self.clear_audio();
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
    fn open_path(&mut self, ctx: &egui::Context, path: &Path) {
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

    /// Carga la ROM de demostración incluida.
    fn load_demo(&mut self, ctx: &egui::Context) {
        let demo = RomFile::from_bytes("demo.nes", nes_core::demo_rom());
        self.load_rom(ctx, &demo);
    }

    /// Extrae el cartucho y deja la pantalla vacía.
    fn close_rom(&mut self, ctx: &egui::Context) {
        self.emulator.unload_rom();
        self.screen.clear();
        self.clear_audio();
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(APP_NAME.to_owned()));
    }

    /// Descarta el sonido pendiente de la ROM anterior.
    fn clear_audio(&self) {
        if let Some(audio) = &self.audio {
            audio.clear();
        }
    }

    /// Carga el primer archivo soltado sobre la ventana, si lo hay.
    #[cfg(not(target_arch = "wasm32"))]
    fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|input| input.raw.dropped_files.first().cloned());
        if let Some(file) = dropped {
            self.open_path(ctx, file.path());
        }
    }

    /// Pasa al emulador el estado de teclado y mandos.
    fn apply_input(&mut self, ctx: &egui::Context) {
        let keyboard = self.config.key_bindings.read(ctx);
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
        if self.emulator.rom().is_none() {
            return;
        }

        let now = ctx.input(|input| input.time);
        let mut frame_rate = self.emulator.region().frame_rate();
        if let Some(audio) = &self.audio {
            audio.set_volume(f32::from(self.config.volume) / 100.0, self.config.muted);
            // El reloj de la tarjeta de sonido y el de la pantalla nunca coinciden del
            // todo. Con la cola de audio baja se emula un poco más rápido y con la cola
            // alta un poco más lento, para que ni se vacíe ni se acumule retraso.
            let error = f64::from(audio.fill_error()).clamp(-1.0, 1.0);
            frame_rate *= 1.0 - error * MAX_RATE_CORRECTION;
        }
        let frames = self.clock.frames_due(now, frame_rate);
        self.apply_input(ctx);

        for _ in 0..frames {
            if let Err(err) = self.emulator.run_frame() {
                self.close_rom(ctx);
                self.error = Some(err.to_string());
                return;
            }
            if let Some(audio) = &self.audio {
                audio.push(self.emulator.audio_samples());
            }
        }
        if frames > 0 {
            self.screen.upload(ctx, self.emulator.frame());
        }
    }

    /// Barra de menús.
    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("Archivo", |ui| {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let open = egui::Button::new("Abrir ROM…")
                        .shortcut_text(ctx.format_shortcut(&OPEN_SHORTCUT));
                    if ui.add(open).clicked() {
                        self.open_dialog(&ctx);
                    }
                    self.recent_menu(ui, &ctx);
                    ui.separator();
                }
                if ui.button("Cargar demo").clicked() {
                    self.load_demo(&ctx);
                }
                let has_rom = self.emulator.rom().is_some();
                if ui
                    .add_enabled(has_rom, egui::Button::new("Cerrar ROM"))
                    .clicked()
                {
                    self.close_rom(&ctx);
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    ui.separator();
                    if ui.button("Salir").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                }
            });
            ui.menu_button("Ver", |ui| {
                ui.checkbox(&mut self.config.integer_scaling, "Escalado entero");
            });
            ui.menu_button("Audio", |ui| {
                if let Some(error) = &self.audio_error {
                    ui.colored_label(ui.visuals().warn_fg_color, "Sin sonido");
                    ui.weak(error);
                    return;
                }
                let mute = egui::Checkbox::new(&mut self.config.muted, "Silenciar");
                ui.add(mute)
                    .on_hover_text(ctx.format_shortcut(&MUTE_SHORTCUT));
                ui.add_enabled(
                    !self.config.muted,
                    egui::Slider::new(&mut self.config.volume, 0..=100)
                        .text("Volumen")
                        .suffix(" %"),
                );
            });
            ui.menu_button("Ayuda", |ui| {
                if ui.button("Controles").clicked() {
                    self.show_controls = true;
                }
            });
        });
    }

    /// Submenú con las ROMs abiertas recientemente.
    #[cfg(not(target_arch = "wasm32"))]
    fn recent_menu(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.add_enabled_ui(!self.config.recent_roms.is_empty(), |ui| {
            ui.menu_button("Recientes", |ui| {
                let mut chosen = None;
                for path in self.config.recent_roms.iter() {
                    let name = path
                        .file_name()
                        .map_or_else(|| path.to_string_lossy(), |name| name.to_string_lossy());
                    if ui
                        .button(name)
                        .on_hover_text(path.to_string_lossy())
                        .clicked()
                    {
                        chosen = Some(path.to_path_buf());
                    }
                }
                ui.separator();
                if ui.button("Vaciar lista").clicked() {
                    self.config.recent_roms.clear();
                }
                if let Some(path) = chosen {
                    self.open_path(ctx, &path);
                }
            });
        });
    }

    /// Ventana de ayuda con las teclas de cada jugador y los mandos conectados.
    fn controls_window(&mut self, ctx: &egui::Context) {
        let mut open = self.show_controls;
        egui::Window::new("Controles")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                egui::Grid::new("controls").striped(true).show(ui, |ui| {
                    ui.strong("Botón");
                    ui.strong("Jugador 1");
                    ui.strong("Jugador 2");
                    ui.end_row();
                    for button in Button::ALL {
                        ui.label(button_label(button));
                        for player in PLAYERS {
                            let key = self.config.key_bindings.map(player).key(button);
                            ui.monospace(key.symbol_or_name());
                        }
                        ui.end_row();
                    }
                });
                ui.separator();
                let names = self.gamepads.names();
                if names.is_empty() {
                    ui.weak("Ningún mando conectado.");
                }
                for (player, name) in PLAYERS.iter().zip(&names) {
                    ui.label(format!("{}: {name}", player_label(*player)));
                }
            });
        self.show_controls = open;
    }

    /// Ventana con el último error, hasta que el usuario la cierra.
    fn error_window(&mut self, ctx: &egui::Context) {
        let Some(message) = &self.error else {
            return;
        };
        let mut dismissed = false;
        egui::Window::new("Error")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_max_width(420.0);
                ui.label(message);
                ui.add_space(8.0);
                dismissed = ui.button("Aceptar").clicked();
            });
        if dismissed {
            self.error = None;
        }
    }
}

/// Nombre de un botón para la interfaz.
const fn button_label(button: Button) -> &'static str {
    match button {
        Button::Up => "Arriba",
        Button::Down => "Abajo",
        Button::Left => "Izquierda",
        Button::Right => "Derecha",
        Button::A => "A",
        Button::B => "B",
        Button::Select => "Select",
        Button::Start => "Start",
    }
}

/// Nombre de un jugador para la interfaz.
const fn player_label(player: Player) -> &'static str {
    match player {
        Player::One => "Jugador 1",
        Player::Two => "Jugador 2",
    }
}

impl eframe::App for NesApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(not(target_arch = "wasm32"))]
        if ctx.input_mut(|input| input.consume_shortcut(&OPEN_SHORTCUT)) {
            self.open_dialog(ctx);
        }
        if !ctx.egui_wants_keyboard_input()
            && ctx.input_mut(|input| input.consume_shortcut(&MUTE_SHORTCUT))
        {
            self.config.muted = !self.config.muted;
        }
        #[cfg(not(target_arch = "wasm32"))]
        self.handle_dropped_files(ctx);
        self.run_emulation(ctx);
        // La emulación es continua: se pide repintar siempre, sin esperar a eventos.
        ctx.request_repaint();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("menu").show(ui, |ui| self.menu_bar(ui));

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                self.screen.show(ui, self.config.integer_scaling);
                if self.emulator.rom().is_none() {
                    ui.put(
                        ui.max_rect(),
                        egui::Label::new(
                            egui::RichText::new(
                                "Arrastra una ROM (.nes) aquí o usa Archivo → Abrir ROM…",
                            )
                            .color(egui::Color32::GRAY)
                            .size(16.0),
                        ),
                    );
                }
            });

        self.controls_window(ui.ctx());
        self.error_window(ui.ctx());
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        self.config.save(storage);
    }
}
