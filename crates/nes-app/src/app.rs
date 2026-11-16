//! La aplicación: une el emulador con la ventana.

use eframe::egui;
use nes_core::Emulator;

use crate::{clock::FrameClock, video::Screen};

/// Estado de la aplicación.
pub struct NesApp {
    emulator: Emulator,
    screen: Screen,
    clock: FrameClock,
    /// Escalar la imagen solo en múltiplos enteros.
    integer_scaling: bool,
    /// Último error de emulación, que detiene la consola hasta cargar otra ROM.
    error: Option<String>,
}

impl NesApp {
    /// Crea la app con la ROM de demostración cargada.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut emulator = Emulator::new();
        let error = emulator
            .load_rom("demo", &nes_core::demo_rom())
            .err()
            .map(|err| err.to_string());
        Self {
            emulator,
            screen: Screen::default(),
            clock: FrameClock::default(),
            integer_scaling: true,
            error,
        }
    }

    /// Emula los frames que tocan desde el repintado anterior.
    fn run_emulation(&mut self, ctx: &egui::Context) {
        if self.error.is_some() || self.emulator.rom().is_none() {
            return;
        }

        let now = ctx.input(|input| input.time);
        let frame_rate = self.emulator.region().frame_rate();
        let frames = self.clock.frames_due(now, frame_rate);

        for _ in 0..frames {
            if let Err(err) = self.emulator.run_frame() {
                self.error = Some(err.to_string());
                return;
            }
        }
        if frames > 0 {
            self.screen.upload(ctx, self.emulator.frame());
        }
    }
}

impl eframe::App for NesApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.run_emulation(ctx);
        // La emulación es continua: se pide repintar siempre, sin esperar a eventos.
        ctx.request_repaint();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                self.screen.show(ui, self.integer_scaling);
                if let Some(error) = &self.error {
                    ui.put(
                        ui.max_rect().shrink(16.0),
                        egui::Label::new(
                            egui::RichText::new(error).color(egui::Color32::LIGHT_RED),
                        ),
                    );
                }
            });
    }
}
