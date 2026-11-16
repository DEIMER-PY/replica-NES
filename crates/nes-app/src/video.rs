//! Vídeo: la textura con el frame de la NES y su colocación en la ventana.

use eframe::egui::{self, Color32, ColorImage, Rect, TextureHandle, TextureOptions, Vec2};
use nes_core::{FRAME_BYTES, SCREEN_HEIGHT, SCREEN_WIDTH};

/// Tamaño nativo de la imagen de la NES.
pub const NES_SIZE: Vec2 = Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);

/// Ancho de un píxel de la NES respecto a su alto en un televisor NTSC.
///
/// La consola generaba píxeles algo más anchos que altos, así que la imagen de 256×240
/// se veía con proporción cercana a 4:3 en vez de 16:15.
const TV_PIXEL_ASPECT: f32 = 8.0 / 7.0;

/// Cómo se escala la imagen a la ventana.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scaling {
    /// Escalar solo en múltiplos enteros (×1, ×2, ×3...), sin píxeles desiguales.
    pub integer: bool,
    /// Ensanchar la imagen como en un televisor (píxeles 8:7) en vez de píxeles cuadrados.
    pub tv_aspect: bool,
}

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

    /// Descarta la imagen: la pantalla queda en negro hasta el próximo frame.
    pub fn clear(&mut self) {
        self.texture = None;
    }

    /// Dibuja la imagen centrada en el espacio disponible de `ui`.
    pub fn show(&self, ui: &mut egui::Ui, scaling: Scaling) {
        let available = ui.available_rect_before_wrap();
        ui.painter().rect_filled(available, 0.0, Color32::BLACK);

        if let Some(texture) = &self.texture {
            let rect = fit_rect(available, ui.pixels_per_point(), scaling);
            let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            ui.painter().image(texture.id(), rect, uv, Color32::WHITE);
        }
        ui.allocate_rect(available, egui::Sense::hover());
    }
}

/// Calcula dónde se dibuja la imagen dentro de `available`, conservando la proporción.
///
/// Con escalado entero el alto es un múltiplo entero en píxeles físicos (×1, ×2, ×3...).
/// `pixels_per_point` es el factor de escala de la pantalla (por ejemplo 1.5 en un
/// monitor al 150 %).
pub fn fit_rect(available: Rect, pixels_per_point: f32, scaling: Scaling) -> Rect {
    let pixel_aspect = if scaling.tv_aspect {
        TV_PIXEL_ASPECT
    } else {
        1.0
    };
    // Tamaño de la imagen a escala ×1, en puntos.
    let base = Vec2::new(NES_SIZE.x * pixel_aspect, NES_SIZE.y);

    let scale = (available.width() / base.x).min(available.height() / base.y);
    let scale = if scaling.integer {
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
    Rect::from_center_size(available.center(), base * scale)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SQUARE_INTEGER: Scaling = Scaling {
        integer: true,
        tv_aspect: false,
    };
    const SQUARE_FREE: Scaling = Scaling {
        integer: false,
        tv_aspect: false,
    };

    fn area(width: f32, height: f32) -> Rect {
        Rect::from_min_size(egui::pos2(0.0, 0.0), Vec2::new(width, height))
    }

    #[test]
    fn integer_scaling_uses_whole_multiples() {
        let rect = fit_rect(area(800.0, 600.0), 1.0, SQUARE_INTEGER);
        // 800/256 = 3.1 y 600/240 = 2.5: cabe ×2.
        assert_eq!(rect.size(), Vec2::new(512.0, 480.0));
        assert_eq!(rect.center(), egui::pos2(400.0, 300.0));
    }

    #[test]
    fn integer_scaling_counts_physical_pixels() {
        // Al 150 %, 512×480 puntos son 768×720 píxeles físicos: ×3 exacto.
        let rect = fit_rect(area(512.0, 480.0), 1.5, SQUARE_INTEGER);
        assert_eq!(rect.size(), Vec2::new(512.0, 480.0));
    }

    #[test]
    fn free_scaling_fills_the_limiting_side() {
        let rect = fit_rect(area(800.0, 600.0), 1.0, SQUARE_FREE);
        assert_eq!(rect.size(), Vec2::new(640.0, 600.0));
    }

    #[test]
    fn shrinks_when_window_is_tiny() {
        let rect = fit_rect(area(128.0, 200.0), 1.0, SQUARE_INTEGER);
        assert_eq!(rect.size(), Vec2::new(128.0, 120.0));
    }

    #[test]
    fn tv_aspect_widens_the_picture() {
        let scaling = Scaling {
            integer: true,
            tv_aspect: true,
        };
        let rect = fit_rect(area(1000.0, 480.0), 1.0, scaling);
        // Alto ×2 exacto; ancho 256 × 2 × 8/7.
        assert_eq!(rect.height(), 480.0);
        assert!((rect.width() - 585.14).abs() < 0.01, "{}", rect.width());
        assert!((rect.width() / rect.height() - 1.219).abs() < 0.001);
    }

    #[test]
    fn tv_aspect_respects_a_narrow_window() {
        let scaling = Scaling {
            integer: false,
            tv_aspect: true,
        };
        let rect = fit_rect(area(292.57, 1000.0), 1.0, scaling);
        assert!((rect.width() - 292.57).abs() < 0.01);
        assert!((rect.height() - 240.0).abs() < 0.01);
    }
}
