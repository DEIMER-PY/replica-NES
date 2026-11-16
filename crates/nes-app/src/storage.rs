//! Guardado en disco de partidas (RAM con batería) y estados (save states).
//!
//! Los archivos viven en la carpeta de datos de la app, separados de las ROMs:
//!
//! ```text
//! <datos>/saves/<rom>.sav             partida guardada por el propio juego
//! <datos>/states/<rom>.<ranura>.state estado guardado por el usuario
//! ```

use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// Qué se guarda.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveKind {
    /// RAM con batería del cartucho.
    Battery,
    /// Estado de la consola en una ranura.
    State(u8),
}

/// Almacén de partidas y estados en una carpeta.
#[derive(Debug, Clone)]
pub struct SaveStore {
    /// Carpeta raíz, o `None` si el sistema no ofrece carpeta de datos.
    dir: Option<PathBuf>,
}

impl SaveStore {
    /// Almacén en la carpeta de datos de la app (junto a su configuración).
    pub fn for_app(app_name: &str) -> Self {
        Self {
            dir: eframe::storage_dir(app_name),
        }
    }

    /// Almacén en una carpeta concreta.
    #[cfg(test)]
    pub fn in_dir(dir: &Path) -> Self {
        Self {
            dir: Some(dir.to_path_buf()),
        }
    }

    /// Ruta del archivo de `kind` para la ROM `rom_id`.
    fn path(&self, kind: SaveKind, rom_id: &str) -> Result<PathBuf, String> {
        let dir = self
            .dir
            .as_deref()
            .ok_or("No hay carpeta de datos donde guardar")?;
        Ok(match kind {
            SaveKind::Battery => dir.join("saves").join(format!("{rom_id}.sav")),
            SaveKind::State(slot) => dir.join("states").join(format!("{rom_id}.{slot}.state")),
        })
    }

    /// Guarda `data`, reemplazando lo que hubiera.
    ///
    /// # Errors
    ///
    /// Devuelve un mensaje para el usuario si no se puede escribir.
    pub fn write(&self, kind: SaveKind, rom_id: &str, data: &[u8]) -> Result<(), String> {
        let path = self.path(kind, rom_id)?;
        write_atomic(&path, data)
            .map_err(|err| format!("No se pudo guardar {}: {err}", path.display()))
    }

    /// Lee lo guardado, o `None` si todavía no existe.
    ///
    /// # Errors
    ///
    /// Devuelve un mensaje para el usuario si el archivo existe pero no se puede leer.
    pub fn read(&self, kind: SaveKind, rom_id: &str) -> Result<Option<Vec<u8>>, String> {
        let path = self.path(kind, rom_id)?;
        match fs::read(&path) {
            Ok(data) => Ok(Some(data)),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(format!("No se pudo leer {}: {err}", path.display())),
        }
    }
}

/// Escribe a un archivo temporal y lo renombra: si la app se cierra a mitad, el
/// archivo anterior queda intacto en vez de a medio escribir.
fn write_atomic(path: &Path, data: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("tmp");
    fs::write(&temp, data)?;
    fs::rename(&temp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Carpeta temporal propia de un test, que se borra al terminar.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(name: &str) -> Self {
            let dir = std::env::temp_dir().join(format!("replica-nes-test-{name}"));
            let _ = fs::remove_dir_all(&dir);
            Self(dir)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn missing_save_reads_as_none() {
        let dir = TempDir::new("missing");
        let store = SaveStore::in_dir(&dir.0);
        assert_eq!(store.read(SaveKind::Battery, "juego"), Ok(None));
    }

    #[test]
    fn write_then_read_round_trips() {
        let dir = TempDir::new("roundtrip");
        let store = SaveStore::in_dir(&dir.0);
        store.write(SaveKind::Battery, "juego", &[1, 2, 3]).unwrap();
        store.write(SaveKind::Battery, "juego", &[4, 5]).unwrap();
        assert_eq!(store.read(SaveKind::Battery, "juego"), Ok(Some(vec![4, 5])));
        assert!(dir.0.join("saves").join("juego.sav").is_file());
        assert!(!dir.0.join("saves").join("juego.tmp").exists());
    }

    #[test]
    fn slots_and_roms_are_independent() {
        let dir = TempDir::new("slots");
        let store = SaveStore::in_dir(&dir.0);
        store.write(SaveKind::State(1), "a", b"uno").unwrap();
        store.write(SaveKind::State(2), "a", b"dos").unwrap();

        assert_eq!(
            store.read(SaveKind::State(1), "a"),
            Ok(Some(b"uno".to_vec()))
        );
        assert_eq!(
            store.read(SaveKind::State(2), "a"),
            Ok(Some(b"dos".to_vec()))
        );
        assert_eq!(store.read(SaveKind::State(1), "b"), Ok(None));
        assert_eq!(store.read(SaveKind::Battery, "a"), Ok(None));
    }

    #[test]
    fn store_without_directory_reports_error() {
        let store = SaveStore { dir: None };
        assert!(store.write(SaveKind::Battery, "juego", &[1]).is_err());
        assert!(store.read(SaveKind::Battery, "juego").is_err());
    }
}
