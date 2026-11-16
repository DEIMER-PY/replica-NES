//! Piezas exclusivas de la versión web (WebAssembly).
//!
//! En el navegador leer un archivo es asíncrono: no se puede bloquear esperando al
//! usuario como hace el diálogo nativo. Las ROMs elegidas o soltadas se leen en una
//! tarea aparte y llegan a la app por un canal ([`RomInbox`]), que esta consulta en
//! cada repintado.

use std::sync::mpsc::{Receiver, Sender, channel};

use eframe::egui::DroppedFileHandle;

use crate::rom::RomFile;

/// Buzón de ROMs que terminan de leerse en segundo plano.
pub struct RomInbox {
    sender: Sender<Result<RomFile, String>>,
    receiver: Receiver<Result<RomFile, String>>,
}

impl Default for RomInbox {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self { sender, receiver }
    }
}

impl RomInbox {
    /// Siguiente ROM ya leída (o el error al leerla), si hay alguna.
    pub fn poll(&self) -> Option<Result<RomFile, String>> {
        self.receiver.try_recv().ok()
    }

    /// Abre el selector de archivos del navegador y lee la ROM elegida.
    pub fn pick_file(&self) {
        let sender = self.sender.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let picked = rfd::AsyncFileDialog::new()
                .add_filter("ROM de NES", &["nes"])
                .pick_file()
                .await;
            if let Some(file) = picked {
                let rom = RomFile::from_bytes(&file.file_name(), file.read().await);
                // Si la app ya se cerró no hay quien reciba; no es un error.
                let _ = sender.send(Ok(rom));
            }
        });
    }

    /// Lee un archivo soltado sobre la página.
    pub fn read_dropped(&self, file: DroppedFileHandle) {
        let sender = self.sender.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let name = file.path().to_string_lossy().into_owned();
            let result = file
                .bytes_async()
                .await
                .map(|data| RomFile::from_bytes(&name, data))
                .map_err(|err| format!("No se pudo leer {name}: {err}"));
            let _ = sender.send(result);
        });
    }
}

/// Quita el aviso "Cargando…" de la página cuando la app ya está en marcha.
pub fn hide_loading_message() {
    let loading = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("loading"));
    if let Some(loading) = loading {
        loading.remove();
    }
}
