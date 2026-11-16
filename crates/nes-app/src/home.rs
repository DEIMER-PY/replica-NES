//! Pantalla de inicio: lo que se ve mientras no hay ningún juego cargado.
//!
//! Ofrece las formas de abrir un juego (botón, carpeta de juegos, arrastrar) y lista la
//! biblioteca y las ROMs recientes para abrirlas con un clic.

use std::path::PathBuf;

use eframe::egui::{self, Color32, RichText};

use crate::app::{APP_NAME, Action, NesApp};

/// Ancho máximo del contenido, para que no se estire en ventanas grandes.
const MAX_WIDTH: f32 = 620.0;
/// Margen entre el contenido y el borde de la ventana.
const MARGIN: f32 = 20.0;
/// Fondo de la pantalla de inicio.
const BACKGROUND: Color32 = Color32::from_rgb(24, 24, 30);

/// Lo que el usuario ha pedido en la pantalla de inicio durante este repintado.
enum Request {
    Perform(Action),
    Open(PathBuf),
}

impl NesApp {
    /// Dibuja la pantalla de inicio en el espacio disponible de `ui`.
    pub(crate) fn home(&mut self, ui: &mut egui::Ui) {
        let area = ui.available_rect_before_wrap();
        ui.painter().rect_filled(area, 0.0, BACKGROUND);

        let size = egui::vec2(
            (area.width() - 2.0 * MARGIN).min(MAX_WIDTH),
            area.height() - 2.0 * MARGIN,
        );
        let content = egui::Rect::from_center_size(area.center(), size);

        let mut request = None;
        ui.scope_builder(egui::UiBuilder::new().max_rect(content), |ui| {
            ui.label(RichText::new(APP_NAME).size(30.0).strong());
            ui.weak("Emulador de NES");
            ui.add_space(14.0);

            self.open_buttons(ui, &mut request);
            ui.add_space(6.0);
            ui.weak("También puedes arrastrar un archivo .nes a esta ventana.");
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(6.0);

            // La biblioteca y los recientes son rutas de disco, que en la web no existen.
            #[cfg(not(target_arch = "wasm32"))]
            self.game_lists(ui, &mut request);
        });
        ui.allocate_rect(area, egui::Sense::hover());

        let ctx = ui.ctx().clone();
        match request {
            Some(Request::Perform(action)) => self.perform(&ctx, action),
            Some(Request::Open(path)) => self.open_path(&ctx, &path),
            None => {}
        }
    }

    /// Botones principales: abrir un archivo, elegir la carpeta de juegos y la demo.
    fn open_buttons(&self, ui: &mut egui::Ui, request: &mut Option<Request>) {
        ui.horizontal_wrapped(|ui| {
            let mut big_button = |ui: &mut egui::Ui, text: &str, hint: &str, action: Action| {
                let button = egui::Button::new(RichText::new(text).size(16.0))
                    .min_size(egui::vec2(0.0, 34.0));
                if ui.add(button).on_hover_text(hint).clicked() {
                    *request = Some(Request::Perform(action));
                }
            };

            let open_hint = Action::OpenRom.shortcut().map_or_else(
                || "Elige un archivo .nes".to_owned(),
                |shortcut| {
                    format!(
                        "Elige un archivo .nes ({})",
                        ui.ctx().format_shortcut(&shortcut)
                    )
                },
            );
            big_button(ui, "Abrir ROM…", &open_hint, Action::OpenRom);
            #[cfg(not(target_arch = "wasm32"))]
            big_button(
                ui,
                "Carpeta de juegos…",
                "Elige la carpeta donde guardas tus juegos para verlos en la biblioteca",
                Action::ChooseRomDir,
            );
            big_button(
                ui,
                "Probar la demo",
                "ROM de prueba incluida: pantalla azul, y roja con sonido al mantener el botón A",
                Action::LoadDemo,
            );
        });
    }

    /// Biblioteca de la carpeta de juegos y, debajo, las ROMs recientes.
    fn game_lists(&self, ui: &mut egui::Ui, request: &mut Option<Request>) {
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                self.library_list(ui, request);
                if !self.config.recent_roms.is_empty() {
                    ui.add_space(12.0);
                    ui.strong("Recientes");
                    ui.add_space(2.0);
                    for path in self.config.recent_roms.iter() {
                        let name = path
                            .file_stem()
                            .map_or_else(|| path.to_string_lossy(), |name| name.to_string_lossy());
                        if game_row(ui, &name, "")
                            .on_hover_text(path.to_string_lossy())
                            .clicked()
                        {
                            *request = Some(Request::Open(path.to_path_buf()));
                        }
                    }
                }
            });
    }

    /// Juegos de la carpeta elegida.
    fn library_list(&self, ui: &mut egui::Ui, request: &mut Option<Request>) {
        let Some(dir) = &self.config.rom_dir else {
            ui.strong("Biblioteca");
            ui.label("Pulsa «Carpeta de juegos…» y elige dónde guardas tus archivos .nes.");
            ui.weak("Aparecerán aquí para abrirlos con un clic.");
            return;
        };

        ui.horizontal(|ui| {
            ui.strong(format!("Biblioteca ({})", self.library.len()));
            if ui
                .small_button("Actualizar")
                .on_hover_text("Vuelve a leer la carpeta")
                .clicked()
            {
                *request = Some(Request::Perform(Action::RefreshLibrary));
            }
        });
        ui.weak(dir.to_string_lossy());
        ui.add_space(2.0);

        if self.library.is_empty() {
            ui.label("No hay archivos .nes en esta carpeta.");
        }
        for entry in &self.library {
            let size = format!("{} KB", entry.size_kb());
            if game_row(ui, &entry.name, &size)
                .on_hover_text(entry.path.to_string_lossy())
                .clicked()
            {
                *request = Some(Request::Open(entry.path.clone()));
            }
        }
    }

    /// Aviso a pantalla completa mientras se arrastra un archivo sobre la ventana.
    pub(crate) fn drop_hint(ui: &egui::Ui) {
        let hovering = ui.ctx().input(|input| !input.raw.hovered_files.is_empty());
        if !hovering {
            return;
        }
        let area = ui.max_rect();
        let layer = egui::LayerId::new(egui::Order::Foreground, egui::Id::unique("drop-hint"));
        let painter = ui.ctx().layer_painter(layer);
        painter.rect_filled(area, 0.0, Color32::from_black_alpha(200));
        painter.text(
            area.center(),
            egui::Align2::CENTER_CENTER,
            "Suelta el archivo para abrirlo",
            egui::FontId::proportional(26.0),
            Color32::WHITE,
        );
    }
}

/// Fila de un juego: ocupa todo el ancho, con el nombre a la izquierda y `detail` a la
/// derecha.
fn game_row(ui: &mut egui::Ui, name: &str, detail: &str) -> egui::Response {
    let button = egui::Button::new(name)
        .shortcut_text(detail)
        .frame_when_inactive(false);
    ui.add_sized([ui.available_width(), 26.0], button)
}
