//! replica-NES: emulador de NES para escritorio y web.
//!
//! Escritorio: `nes-app [ROM]`. Sin argumentos arranca con la ROM de demostración.
//! Web: se compila a WebAssembly con `trunk` (ver `index.html`).

// En Windows, sin consola al abrir la app en modo release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// La versión web no usa lo que depende del disco (rutas, ROMs recientes...).
#![cfg_attr(target_arch = "wasm32", allow(dead_code))]

mod app;
mod audio;
mod clock;
mod config;
mod home;
mod input;
mod rom;
mod settings;
#[cfg(not(target_arch = "wasm32"))]
mod storage;
#[cfg(target_arch = "wasm32")]
#[path = "storage_web.rs"]
mod storage;
mod ui;
mod video;
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    native::run()
}

#[cfg(target_arch = "wasm32")]
fn main() {
    wasm::run();
}

/// Arranque en escritorio: una ventana nativa.
#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::path::PathBuf;

    use eframe::egui;

    /// Icono de la ventana y de la barra de tareas (en Windows el `.exe` lo lleva
    /// además incrustado, ver `build.rs`).
    pub(crate) fn icon() -> egui::IconData {
        eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
            .expect("assets/icon.png es un PNG válido")
    }

    use crate::{app, video};

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

    pub fn run() -> eframe::Result {
        // Primer argumento: ruta de una ROM (también al arrastrar un .nes sobre el ejecutable).
        let initial_rom = std::env::args_os().nth(1).map(PathBuf::from);

        let window_size = video::NES_SIZE * INITIAL_SCALE + egui::vec2(0.0, MENU_HEIGHT);
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title(app::APP_NAME)
                .with_inner_size(window_size)
                .with_min_inner_size(video::NES_SIZE + egui::vec2(0.0, MENU_HEIGHT))
                .with_drag_and_drop(true)
                .with_icon(icon()),
            wgpu_options: wgpu_options(),
            ..Default::default()
        };
        eframe::run_native(
            app::APP_NAME,
            options,
            Box::new(|cc| Ok(Box::new(app::NesApp::new(cc, initial_rom)))),
        )
    }
}

/// Arranque en la web: la app se dibuja en el `<canvas>` de `index.html`.
#[cfg(target_arch = "wasm32")]
mod wasm {
    use eframe::wasm_bindgen::JsCast as _;

    use crate::{app, web};

    /// `id` del `<canvas>` en `index.html`.
    const CANVAS_ID: &str = "nes-canvas";

    pub fn run() {
        wasm_bindgen_futures::spawn_local(async {
            let canvas = web_sys::window()
                .and_then(|window| window.document())
                .and_then(|document| document.get_element_by_id(CANVAS_ID))
                .and_then(|element| element.dyn_into::<web_sys::HtmlCanvasElement>().ok())
                .expect("index.html debe tener un <canvas id=\"nes-canvas\">");

            let started = eframe::WebRunner::new()
                .start(
                    canvas,
                    eframe::WebOptions::default(),
                    Box::new(|cc| Ok(Box::new(app::NesApp::new(cc, None)))),
                )
                .await;

            match started {
                Ok(()) => web::hide_loading_message(),
                Err(err) => panic!("no se pudo iniciar la app: {err:?}"),
            }
        });
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    #[test]
    fn el_icono_es_valido() {
        let icon = crate::native::icon();
        assert_eq!((icon.width, icon.height), (256, 256));
        assert_eq!(icon.rgba.len(), 256 * 256 * 4);
    }
}
