//! Vídeo: la textura con el frame de la NES y su colocación en la ventana.

use eframe::egui::{self, Color32, ColorImage, Rect, TextureHandle, TextureOptions, Vec2};
use nes_core::{FRAME_BYTES, SCREEN_HEIGHT, SCREEN_WIDTH};

/// Tamaño nativo de la imagen de la NES.
pub const NES_SIZE: Vec2 = Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);

/// Textura de GPU con el último frame emulado.
#[derive(Default)]
pub struct Screen {
    texture: Option<TextureHandle>,
}

impl Screen {
    /// Sube un frame RGBA a la GPU.
    pub fn upload(&mut self, ctx: &egui::Context, frame: &[u8; FRAME_BYTES]) {
        let image = ColorImage::from_rgba_unmultiplied([SCREEN_WIDTH, SCREEN_HEIGHT], frame);
        // NEAREST: sin suavizado, para que los píxeles se vean nítidos al escalar.
        match &mut self.texture {
            Some(texture) => texture.set(image, TextureOptions::NEAREST),
            None => {
                self.texture = Some(ctx.load_texture("nes-frame", image, TextureOptions::NEAREST));
            }
        }
    }

    /// Dibuja la imagen centrada en el espacio disponible de `ui`.
    pub fn show(&self, ui: &mut egui::Ui, integer_scaling: bool) {
        let available = ui.available_rect_before_wrap();
        ui.painter().rect_filled(available, 0.0, Color32::BLACK);

        if let Some(texture) = &self.texture {
            let rect = fit_rect(available, ui.pixels_per_point(), integer_scaling);
            let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            ui.painter().image(texture.id(), rect, uv, Color32::WHITE);
        }
        ui.allocate_rect(available, egui::Sense::hover());
    }
}

/// Calcula dónde se dibuja la imagen dentro de `available`, conservando la proporción.
///
/// Con `integer_scaling` la escala es un múltiplo entero en píxeles físicos (×1, ×2,
/// ×3...), que evita píxeles de tamaños desiguales. `pixels_per_point` es el factor
/// de escala de la pantalla (por ejemplo 1.5 en un monitor al 150 %).
pub fn fit_rect(available: Rect, pixels_per_point: f32, integer_scaling: bool) -> Rect {
    let scale = (available.width() / NES_SIZE.x).min(available.height() / NES_SIZE.y);
    let scale = if integer_scaling {
        let physical = (scale * pixels_per_point).floor();
        if physical >= 1.0 {
            physical / pixels_per_point
        } else {
            // La ventana es más pequeña que la imagen a ×1: se reduce para que quepa.
            scale
        }
    } else {
        scale
    };
    Rect::from_center_size(available.center(), NES_SIZE * scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn area(width: f32, height: f32) -> Rect {
        Rect::from_min_size(egui::pos2(0.0, 0.0), Vec2::new(width, height))
    }

    #[test]
    fn integer_scaling_uses_whole_multiples() {
        let rect = fit_rect(area(800.0, 600.0), 1.0, true);
        // 800/256 = 3.1 y 600/240 = 2.5: cabe ×2.
        assert_eq!(rect.size(), Vec2::new(512.0, 480.0));
        assert_eq!(rect.center(), egui::pos2(400.0, 300.0));
    }

    #[test]
    fn integer_scaling_counts_physical_pixels() {
        // Al 150 %, 512×480 puntos son 768×720 píxeles físicos: ×3 exacto.
        let rect = fit_rect(area(512.0, 480.0), 1.5, true);
        assert_eq!(rect.size(), Vec2::new(512.0, 480.0));
    }

    #[test]
    fn free_scaling_fills_the_limiting_side() {
        let rect = fit_rect(area(800.0, 600.0), 1.0, false);
        assert_eq!(rect.size(), Vec2::new(640.0, 600.0));
    }

    #[test]
    fn shrinks_when_window_is_tiny() {
        let rect = fit_rect(area(128.0, 200.0), 1.0, true);
        assert_eq!(rect.size(), Vec2::new(128.0, 120.0));
    }
}
