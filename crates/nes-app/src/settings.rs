//! Ventana de ajustes: controles, vídeo y audio.

use eframe::egui::{self, Key};
use nes_core::{Button, Player};

use crate::{
    app::{Action, NesApp},
    input::{KeyBindings, PLAYERS},
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
    /// Botón que espera a que el usuario pulse su nueva tecla.
    capturing: Option<(Player, Button)>,
    /// Aviso sobre el último intento de asignar una tecla.
    message: Option<String>,
}

impl SettingsWindow {
    /// Si se está esperando una tecla para asignarla a un botón.
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

        let mut open = self.settings.open;
        egui::Window::new("Ajustes")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
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

    fn controls_tab(&mut self, ui: &mut egui::Ui) {
        ui.label("Pulsa una tecla de la tabla para cambiarla.");
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
                        let capturing = self.settings.capturing == Some((player, button));
                        let text = if capturing {
                            "Pulsa una tecla…"
                        } else {
                            self.config.key_bindings.map(player).key(button).name()
                        };
                        if ui.selectable_label(capturing, text).clicked() {
                            self.settings.capturing = (!capturing).then_some((player, button));
                            self.settings.message = None;
                        }
                    }
                    ui.end_row();
                }
            });

        if self.settings.is_capturing() {
            ui.weak("Escape cancela.");
        }
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
        ui.strong("Mandos");
        let names = self.gamepads.names();
        if names.is_empty() {
            ui.weak("Ningún mando conectado.");
        }
        for (player, name) in PLAYERS.iter().zip(&names) {
            ui.label(format!("{}: {name}", player_label(*player)));
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
