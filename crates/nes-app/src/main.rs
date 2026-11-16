//! replica-NES: emulador de NES para escritorio.

// En Windows, sin consola al abrir la app en modo release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod clock;
mod video;

use eframe::egui;

/// Escala inicial de la ventana respecto a la imagen de la NES.
const INITIAL_SCALE: f32 = 3.0;

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
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("replica-NES")
            .with_inner_size(video::NES_SIZE * INITIAL_SCALE)
            .with_min_inner_size(video::NES_SIZE),
        wgpu_options: wgpu_options(),
        ..Default::default()
    };
    eframe::run_native(
        "replica-NES",
        options,
        Box::new(|cc| Ok(Box::new(app::NesApp::new(cc)))),
    )
}
