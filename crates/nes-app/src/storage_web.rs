//! Guardado de partidas y estados en el navegador.
//!
//! Es la versión web de `storage.rs`, con la misma interfaz. En el navegador no hay
//! sistema de archivos, así que los datos van a `localStorage` codificados en base64
//! (solo admite texto), bajo claves como `replica-nes/states/<rom>.1`.
//!
//! `localStorage` es propio de cada navegador y sitio, y tiene un límite de unos 5 MB.

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

/// Qué se guarda.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveKind {
    /// RAM con batería del cartucho.
    Battery,
    /// Estado de la consola en una ranura.
    State(u8),
}

/// Almacén de partidas y estados en `localStorage`.
#[derive(Debug, Clone)]
pub struct SaveStore;

impl SaveStore {
    /// Almacén del navegador. El nombre de la app no se usa: las claves ya llevan prefijo.
    pub fn for_app(_app_name: &str) -> Self {
        Self
    }

    fn key(kind: SaveKind, rom_id: &str) -> String {
        match kind {
            SaveKind::Battery => format!("replica-nes/saves/{rom_id}"),
            SaveKind::State(slot) => format!("replica-nes/states/{rom_id}.{slot}"),
        }
    }

    fn local_storage() -> Result<web_sys::Storage, String> {
        web_sys::window()
            .and_then(|window| window.local_storage().ok().flatten())
            .ok_or_else(|| "El navegador no permite guardar datos en este sitio".to_owned())
    }

    /// Guarda `data`, reemplazando lo que hubiera.
    ///
    /// # Errors
    ///
    /// Devuelve un mensaje para el usuario si el navegador rechaza el guardado, por
    /// ejemplo por falta de espacio o en modo privado.
    pub fn write(&self, kind: SaveKind, rom_id: &str, data: &[u8]) -> Result<(), String> {
        Self::local_storage()?
            .set_item(&Self::key(kind, rom_id), &BASE64.encode(data))
            .map_err(|_| {
                "No se pudo guardar: el almacenamiento del navegador está lleno".to_owned()
            })
    }

    /// Lee lo guardado, o `None` si todavía no existe.
    ///
    /// # Errors
    ///
    /// Devuelve un mensaje para el usuario si el dato existe pero está dañado.
    pub fn read(&self, kind: SaveKind, rom_id: &str) -> Result<Option<Vec<u8>>, String> {
        let stored = Self::local_storage()?
            .get_item(&Self::key(kind, rom_id))
            .map_err(|_| "No se pudo leer el almacenamiento del navegador".to_owned())?;
        stored
            .map(|text| {
                BASE64
                    .decode(text)
                    .map_err(|_| "El dato guardado en el navegador está dañado".to_owned())
            })
            .transpose()
    }
}
