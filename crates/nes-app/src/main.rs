//! replica-NES: emulador de NES para escritorio.
//!
//! Uso: `nes-app [ROM]`. Sin argumentos arranca con la ROM de demostración.

// En Windows, sin consola al abrir la app en modo release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod audio;
mod clock;
mod config;
mod input;
mod rom;
mod settings;
mod storage;
mod ui;
mod video;

use std::path::PathBuf;

use eframe::egui;

/// Escala inicial de la imagen de la NES en la ventana.
const INITIAL_SCALE: f32 = 3.0;
/// Alto aproximado de la barra de menús, para que la imagen quepa a escala entera.
const MENU_HEIGHT: f32 = 32.0;

/// Configuración del render por GPU.
///
/// En Windows se usa DirectX 12 (con OpenGL de reserva) en lugar de dejar que `wgpu`
/// elija Vulkan: algunos drivers de gráficas integradas Intel cierran la app con una
/// violación de acceso al crear el dispositivo Vulkan. La variable de entorno
/// `WGPU_BACKEND` (`dx12`, `vulkan`, `gl`) sigue teniendo prioridad.
fn wgpu_options() -> eframe::WgpuConfiguration {
    use eframe::egui_wgpu::{WgpuSetup, WgpuSetupCreateNew};
    use eframe::wgpu::Backends;

    let mut setup = WgpuSetupCreateNew::without_display_handle();
    if cfg!(target_os = "windows") && Backends::from_env().is_none() {
        setup.instance_descriptor.backends = Backends::DX12 | Backends::GL;
    }
    eframe::WgpuConfiguration {
        wgpu_setup: WgpuSetup::CreateNew(setup),
        ..Default::default()
    }
}

fn main() -> eframe::Result {
    // Primer argumento: ruta de una ROM (también al arrastrar un .nes sobre el ejecutable).
    let initial_rom = std::env::args_os().nth(1).map(PathBuf::from);

    let window_size = video::NES_SIZE * INITIAL_SCALE + egui::vec2(0.0, MENU_HEIGHT);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(app::APP_NAME)
            .with_inner_size(window_size)
            .with_min_inner_size(video::NES_SIZE + egui::vec2(0.0, MENU_HEIGHT))
            .with_drag_and_drop(true),
        wgpu_options: wgpu_options(),
        ..Default::default()
    };
    eframe::run_native(
        app::APP_NAME,
        options,
        Box::new(|cc| Ok(Box::new(app::NesApp::new(cc, initial_rom)))),
    )
}
