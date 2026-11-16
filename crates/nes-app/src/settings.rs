//! Ventana de ajustes: controles, vídeo y audio.

use eframe::egui::{self, Key};
use nes_core::{Button, Player};

use crate::{
    app::{Action, NesApp},
    input::{KeyBindings, PLAYERS, PadBindings},
};

/// Pestañas de la ventana de ajustes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Controls,
    Video,
    Audio,
}

/// Estado de la ventana de ajustes (no se guarda entre sesiones).
#[derive(Debug, Default)]
pub struct SettingsWindow {
    pub open: bool,
    pub tab: Tab,
    /// Botón que espera a que el usuario pulse su nueva tecla o botón de mando.
    capturing: Option<Target>,
    /// Aviso sobre el último intento de asignar una tecla.
    message: Option<String>,
}

/// Qué se está reasignando.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Target {
    /// La tecla de un botón de la NES.
    Key(Player, Button),
    /// El botón de mando de un botón de la NES.
    Pad(Player, Button),
}

impl SettingsWindow {
    /// Si se está esperando una tecla o un botón de mando para asignarlo.
    ///
    /// Mientras tanto, la app no debe tratar las pulsaciones como atajos ni como
    /// botones del juego.
    pub const fn is_capturing(&self) -> bool {
        self.capturing.is_some()
    }

    /// Abre la ventana en una pestaña concreta.
    pub const fn open_at(&mut self, tab: Tab) {
        self.open = true;
        self.tab = tab;
    }
}

/// Resultado de pulsar una tecla mientras se reasigna un botón.
#[derive(Debug, PartialEq, Eq)]
enum Capture {
    /// La tecla se asignó al botón.
    Bound,
    /// Se canceló con `Escape`.
    Cancelled,
    /// La tecla está reservada por la app; se sigue esperando otra.
    Reserved,
}

/// Aplica la tecla pulsada al botón que se estaba reasignando.
fn capture_key(bindings: &mut KeyBindings, target: (Player, Button), key: Key) -> Capture {
    if key == Key::Escape {
        Capture::Cancelled
    } else if Action::is_reserved_key(key) {
        Capture::Reserved
    } else {
        bindings.rebind(target.0, target.1, key);
        Capture::Bound
    }
}

impl NesApp {
    /// Dibuja la ventana de ajustes, si está abierta.
    pub(crate) fn settings_window(&mut self, ctx: &egui::Context) {
        self.capture_pressed_key(ctx);
        self.capture_pressed_pad_button();

        let mut open = self.settings.open;
        egui::Window::new("Ajustes")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            // Si la ventana de la app es baja, el contenido se desplaza en vez de cortarse.
            .vscroll(true)
            .default_width(340.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let tab = &mut self.settings.tab;
                    ui.selectable_value(tab, Tab::Controls, "Controles");
                    ui.selectable_value(tab, Tab::Video, "Vídeo");
                    ui.selectable_value(tab, Tab::Audio, "Audio");
                });
                ui.separator();
                match self.settings.tab {
                    Tab::Controls => self.controls_tab(ui),
                    Tab::Video => self.video_tab(ui),
                    Tab::Audio => self.audio_controls(ui),
                }
            });

        self.settings.open = open;
        if !open {
            // Cerrar la ventana cancela una reasignación a medias.
            self.settings.capturing = None;
            self.settings.message = None;
        }
    }

    /// Si se está reasignando un botón y se ha pulsado una tecla, la aplica.
    fn capture_pressed_key(&mut self, ctx: &egui::Context) {
        let Some(target) = self.settings.capturing else {
            return;
        };
        let pressed = ctx.input(|input| {
            input.events.iter().find_map(|event| match event {
                egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    ..
                } => Some(*key),
                _ => None,
            })
        });
        let Some(key) = pressed else {
            return;
        };
        let target = match target {
            Target::Key(player, button) => (player, button),
            Target::Pad(..) => {
                // Al esperar un botón de mando, el teclado solo sirve para cancelar.
                if key == Key::Escape {
                    self.settings.capturing = None;
                }
                return;
            }
        };
        match capture_key(&mut self.config.key_bindings, target, key) {
            Capture::Bound | Capture::Cancelled => {
                self.settings.capturing = None;
                self.settings.message = None;
            }
            Capture::Reserved => {
                self.settings.message = Some(format!(
                    "La tecla {} está reservada para un atajo de la app.",
                    key.name()
                ));
            }
        }
    }

    /// Si se está reasignando un botón de mando y ese jugador ha pulsado uno, lo aplica.
    fn capture_pressed_pad_button(&mut self) {
        let Some(Target::Pad(player, button)) = self.settings.capturing else {
            return;
        };
        if let Some(input) = self.gamepads.first_pressed(player) {
            self.config
                .pad_bindings
                .map_mut(player)
                .rebind(button, input);
            self.settings.capturing = None;
        }
    }

    fn controls_tab(&mut self, ui: &mut egui::Ui) {
        ui.strong("Teclado");
        ui.label("Haz clic en una tecla de la tabla para cambiarla.");
        ui.add_space(4.0);

        egui::Grid::new("key-bindings")
            .striped(true)
            .min_col_width(90.0)
            .show(ui, |ui| {
                ui.strong("Botón");
                for player in PLAYERS {
                    ui.strong(player_label(player));
                }
                ui.end_row();

                for button in Button::ALL {
                    ui.label(button_label(button));
                    for player in PLAYERS {
                        let target = Target::Key(player, button);
                        let capturing = self.settings.capturing == Some(target);
                        let text = if capturing {
                            "Pulsa una tecla…"
                        } else {
                            self.config.key_bindings.map(player).key(button).name()
                        };
                        if ui.selectable_label(capturing, text).clicked() {
                            self.settings.capturing = (!capturing).then_some(target);
                            self.settings.message = None;
                        }
                    }
                    ui.end_row();
                }
            });

        if let Some(message) = &self.settings.message {
            ui.colored_label(ui.visuals().warn_fg_color, message);
        }

        ui.add_space(4.0);
        let is_default = self.config.key_bindings == KeyBindings::default();
        if ui
            .add_enabled(
                !is_default,
                egui::Button::new("Restaurar teclas por defecto"),
            )
            .clicked()
        {
            self.config.key_bindings = KeyBindings::default();
            self.settings.capturing = None;
            self.settings.message = None;
        }

        ui.separator();
        self.gamepad_section(ui);

        if self.settings.is_capturing() {
            ui.add_space(4.0);
            ui.weak("Escape cancela.");
        }
    }

    /// Mandos conectados y tabla para reasignar sus botones.
    fn gamepad_section(&mut self, ui: &mut egui::Ui) {
        ui.strong("Mandos");
        let names = self.gamepads.names();
        if names.is_empty() {
            ui.weak("Ningún mando conectado.");
        }
        for (player, name) in PLAYERS.iter().zip(&names) {
            ui.label(format!("{}: {name}", player_label(*player)));
        }
        ui.label("Haz clic en un botón de la tabla y pulsa el nuevo en el mando.");
        ui.weak("El stick izquierdo siempre mueve, además de los botones de dirección.");
        ui.add_space(4.0);

        egui::Grid::new("pad-bindings")
            .striped(true)
            .min_col_width(90.0)
            .show(ui, |ui| {
                ui.strong("Botón");
                for player in PLAYERS {
                    ui.strong(player_label(player));
                }
                ui.end_row();

                for button in Button::ALL {
                    ui.label(button_label(button));
                    for (index, player) in PLAYERS.into_iter().enumerate() {
                        let target = Target::Pad(player, button);
                        let capturing = self.settings.capturing == Some(target);
                        let text = if capturing {
                            "Pulsa un botón…".to_owned()
                        } else {
                            self.config.pad_bindings.map(player).input(button).label()
                        };
                        // Sin mando no hay con qué pulsar el botón nuevo.
                        let connected = index < names.len();
                        let cell = ui.add_enabled(
                            connected || capturing,
                            egui::Button::selectable(capturing, text),
                        );
                        if cell.clicked() {
                            self.settings.capturing = (!capturing).then_some(target);
                            self.settings.message = None;
                        }
                    }
                    ui.end_row();
                }
            });

        ui.add_space(4.0);
        let is_default = self.config.pad_bindings == PadBindings::default();
        if ui
            .add_enabled(
                !is_default,
                egui::Button::new("Restaurar botones por defecto"),
            )
            .clicked()
        {
            self.config.pad_bindings = PadBindings::default();
            self.settings.capturing = None;
        }
    }

    fn video_tab(&mut self, ui: &mut egui::Ui) {
        self.video_controls(ui);
    }

    /// Opciones de imagen, compartidas por el menú Ver y la ventana de ajustes.
    pub(crate) fn video_controls(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.config.integer_scaling, "Escalado entero")
            .on_hover_text("Escala solo ×1, ×2, ×3... para que todos los píxeles midan lo mismo.");
        ui.checkbox(&mut self.config.tv_aspect, "Proporción de televisor (8:7)")
            .on_hover_text("Ensancha la imagen como se veía en un televisor de la época.");

        // En la web la pantalla completa la gestiona el propio navegador (F11).
        #[cfg(not(target_arch = "wasm32"))]
        self.fullscreen_control(ui);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn fullscreen_control(&mut self, ui: &mut egui::Ui) {
        let mut fullscreen = ui.input(|input| input.viewport().fullscreen.unwrap_or(false));
        let checkbox = ui.checkbox(&mut fullscreen, "Pantalla completa");
        if checkbox.changed() {
            let ctx = ui.ctx().clone();
            self.perform(&ctx, Action::ToggleFullscreen);
        }
        if let Some(shortcut) = Action::ToggleFullscreen.shortcut() {
            checkbox.on_hover_text(ui.ctx().format_shortcut(&shortcut));
        }
    }

    /// Opciones de sonido, compartidas por el menú Audio y la ventana de ajustes.
    pub(crate) fn audio_controls(&mut self, ui: &mut egui::Ui) {
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

#[cfg(test)]
mod tests {
    use super::*;

    const TARGET: (Player, Button) = (Player::One, Button::A);

    #[test]
    fn free_key_is_bound() {
        let mut bindings = KeyBindings::default();
        assert_eq!(
            capture_key(&mut bindings, TARGET, Key::Space),
            Capture::Bound
        );
        assert_eq!(bindings.find(Key::Space), Some(TARGET));
    }

    #[test]
    fn escape_cancels_without_changes() {
        let mut bindings = KeyBindings::default();
        assert_eq!(
            capture_key(&mut bindings, TARGET, Key::Escape),
            Capture::Cancelled
        );
        assert_eq!(bindings, KeyBindings::default());
    }

    #[test]
    fn app_shortcut_keys_are_rejected() {
        // P pausa, F es el avance rápido, F5 guarda estado: no pueden ser botones.
        for key in [Key::P, Key::F, Key::F5, Key::M, Key::Num1, Key::F11] {
            let mut bindings = KeyBindings::default();
            assert_eq!(
                capture_key(&mut bindings, TARGET, key),
                Capture::Reserved,
                "{key:?}"
            );
            assert_eq!(bindings, KeyBindings::default());
        }
    }

    #[test]
    fn keys_used_with_modifiers_stay_available() {
        // Ctrl+O y Ctrl+R son atajos, pero O y R solas se pueden asignar.
        let mut bindings = KeyBindings::default();
        assert_eq!(capture_key(&mut bindings, TARGET, Key::R), Capture::Bound);
    }
}
