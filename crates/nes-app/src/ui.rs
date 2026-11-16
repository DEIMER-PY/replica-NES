//! Interfaz: barra de menús, pantalla del juego y ventanas.

use eframe::egui::{self, Align2, Color32};
use nes_core::{Button, Player, ResetKind};

use crate::{
    app::{Action, FAST_FORWARD_KEY, NesApp, STATE_SLOTS},
    input::PLAYERS,
};

impl NesApp {
    /// Dibuja toda la interfaz.
    pub(crate) fn show(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("menu").show(ui, |ui| self.menu_bar(ui));

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                self.screen.show(ui, self.config.integer_scaling);
                self.overlays(ui);
            });

        self.controls_window(ui.ctx());
        self.error_window(ui.ctx());
    }

    /// Un elemento de menú que ejecuta `action`, con su atajo a la derecha.
    fn action_button(&mut self, ui: &mut egui::Ui, label: &str, action: Action, enabled: bool) {
        let mut button = egui::Button::new(label);
        if let Some(shortcut) = action.shortcut() {
            button = button.shortcut_text(ui.ctx().format_shortcut(&shortcut));
        }
        if ui.add_enabled(enabled, button).clicked() {
            let ctx = ui.ctx().clone();
            self.perform(&ctx, action);
        }
    }

    fn menu_bar(&mut self, ui: &mut egui::Ui) {
        let has_rom = self.has_rom();
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("Archivo", |ui| {
                self.action_button(ui, "Abrir ROM…", Action::OpenRom, true);
                self.recent_menu(ui);
                ui.separator();
                self.action_button(ui, "Cargar demo", Action::LoadDemo, true);
                self.action_button(ui, "Cerrar ROM", Action::CloseRom, has_rom);
                ui.separator();
                self.action_button(ui, "Salir", Action::Quit, true);
            });

            ui.menu_button("Emulación", |ui| {
                let pause = if self.paused { "Reanudar" } else { "Pausar" };
                self.action_button(ui, pause, Action::TogglePause, has_rom);
                self.action_button(ui, "Reiniciar", Action::Reset(ResetKind::Soft), has_rom);
                self.action_button(
                    ui,
                    "Apagar y encender",
                    Action::Reset(ResetKind::Hard),
                    has_rom,
                );
                ui.separator();
                self.action_button(ui, "Guardar estado", Action::SaveState, has_rom);
                self.action_button(ui, "Cargar estado", Action::LoadState, has_rom);
                ui.menu_button(format!("Ranura {}", self.state_slot), |ui| {
                    for slot in STATE_SLOTS {
                        let selected = self.state_slot == slot;
                        if ui.radio(selected, format!("Ranura {slot}")).clicked() {
                            let ctx = ui.ctx().clone();
                            self.perform(&ctx, Action::SelectSlot(slot));
                        }
                    }
                });
                ui.separator();
                ui.weak(format!(
                    "Avance rápido: mantener {}",
                    FAST_FORWARD_KEY.name()
                ));
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
                let mute = ui.checkbox(&mut self.config.muted, "Silenciar");
                if let Some(shortcut) = Action::ToggleMute.shortcut() {
                    mute.on_hover_text(ui.ctx().format_shortcut(&shortcut));
                }
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
    fn recent_menu(&mut self, ui: &mut egui::Ui) {
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
                    let ctx = ui.ctx().clone();
                    self.open_path(&ctx, &path);
                }
            });
        });
    }

    /// Textos sobre la imagen: ayuda sin ROM, estado de la emulación y avisos.
    fn overlays(&mut self, ui: &egui::Ui) {
        let area = ui.max_rect();
        let painter = ui.painter();

        if !self.has_rom() {
            painter.text(
                area.center(),
                Align2::CENTER_CENTER,
                "Arrastra una ROM (.nes) aquí o usa Archivo → Abrir ROM…",
                egui::FontId::proportional(16.0),
                Color32::GRAY,
            );
        }

        let status = if self.paused {
            Some("PAUSA")
        } else if self.fast_forward {
            Some("RÁPIDO ×4")
        } else {
            None
        };
        if let Some(status) = status {
            label(
                painter,
                area.right_top() + egui::vec2(-12.0, 12.0),
                Align2::RIGHT_TOP,
                status,
            );
        }

        if let Some(message) = self.toast(ui.ctx()) {
            let position = area.left_bottom() + egui::vec2(12.0, -12.0);
            label(painter, position, Align2::LEFT_BOTTOM, message);
        }
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
            .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
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

/// Texto sobre un fondo oscuro, legible encima de cualquier imagen del juego.
fn label(painter: &egui::Painter, position: egui::Pos2, anchor: Align2, text: &str) {
    let galley = painter.layout_no_wrap(
        text.to_owned(),
        egui::FontId::proportional(15.0),
        Color32::WHITE,
    );
    let rect = anchor.anchor_size(position, galley.size());
    painter.rect_filled(
        rect.expand2(egui::vec2(8.0, 5.0)),
        4.0,
        Color32::from_black_alpha(190),
    );
    painter.galley(rect.min, galley, Color32::WHITE);
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
