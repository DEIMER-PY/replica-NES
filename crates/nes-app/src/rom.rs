//! Lectura de archivos de ROM y lista de ROMs recientes.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// Tamaño máximo razonable de una ROM de NES. Evita leer por error un archivo enorme.
const MAX_ROM_BYTES: u64 = 16 * 1024 * 1024;

/// Número de ROMs que se recuerdan en el menú de recientes.
const MAX_RECENT: usize = 8;

/// Una ROM leída, lista para entregar al emulador.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RomFile {
    /// Nombre para mostrar: el del archivo sin extensión.
    pub name: String,
    /// Contenido del archivo.
    pub data: Vec<u8>,
    /// Ruta en disco, si viene de un archivo local (en la web no hay ruta).
    pub path: Option<PathBuf>,
}

impl RomFile {
    /// Lee una ROM del disco.
    ///
    /// # Errors
    ///
    /// Devuelve un mensaje para el usuario si el archivo no existe, no se puede leer
    /// o es demasiado grande para ser una ROM.
    pub fn read(path: &Path) -> Result<Self, String> {
        let shown = path.display();
        let metadata =
            std::fs::metadata(path).map_err(|err| format!("No se pudo abrir {shown}: {err}"))?;
        if !metadata.is_file() {
            return Err(format!("{shown} no es un archivo"));
        }
        if metadata.len() > MAX_ROM_BYTES {
            return Err(format!(
                "{shown} es demasiado grande para ser una ROM de NES"
            ));
        }
        let data = std::fs::read(path).map_err(|err| format!("No se pudo leer {shown}: {err}"))?;
        Ok(Self {
            name: display_name(path),
            data,
            path: Some(path.to_path_buf()),
        })
    }

    /// Crea una ROM a partir de bytes ya en memoria (archivo soltado en la web, demo).
    pub fn from_bytes(file_name: &str, data: Vec<u8>) -> Self {
        Self {
            name: display_name(Path::new(file_name)),
            data,
            path: None,
        }
    }
}

/// Nombre de archivo sin carpeta ni extensión.
fn display_name(path: &Path) -> String {
    path.file_stem().map_or_else(
        || "ROM".to_owned(),
        |stem| stem.to_string_lossy().into_owned(),
    )
}

/// ROMs abiertas recientemente, de la más reciente a la más antigua.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RecentRoms(Vec<PathBuf>);

impl RecentRoms {
    /// Pone `path` al principio de la lista, sin duplicados.
    pub fn push(&mut self, path: &Path) {
        self.remove(path);
        self.0.insert(0, path.to_path_buf());
        self.0.truncate(MAX_RECENT);
    }

    /// Quita `path` de la lista, por ejemplo si el archivo ya no existe.
    pub fn remove(&mut self, path: &Path) {
        self.0.retain(|recent| recent != path);
    }

    /// Vacía la lista.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Rutas de la más reciente a la más antigua.
    pub fn iter(&self) -> impl Iterator<Item = &Path> {
        self.0.iter().map(PathBuf::as_path)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(recent: &RecentRoms) -> Vec<&str> {
        recent.iter().map(|path| path.to_str().unwrap()).collect()
    }

    #[test]
    fn recent_puts_newest_first_without_duplicates() {
        let mut recent = RecentRoms::default();
        recent.push(Path::new("a.nes"));
        recent.push(Path::new("b.nes"));
        recent.push(Path::new("a.nes"));
        assert_eq!(paths(&recent), ["a.nes", "b.nes"]);
    }

    #[test]
    fn recent_is_capped() {
        let mut recent = RecentRoms::default();
        for i in 0..MAX_RECENT + 3 {
            recent.push(Path::new(&format!("{i}.nes")));
        }
        assert_eq!(recent.iter().count(), MAX_RECENT);
        assert_eq!(paths(&recent)[0], "10.nes");
    }

    #[test]
    fn recent_remove_and_clear() {
        let mut recent = RecentRoms::default();
        recent.push(Path::new("a.nes"));
        recent.push(Path::new("b.nes"));
        recent.remove(Path::new("b.nes"));
        assert_eq!(paths(&recent), ["a.nes"]);
        recent.clear();
        assert!(recent.is_empty());
    }

    #[test]
    fn display_name_drops_folder_and_extension() {
        let rom = RomFile::from_bytes("juegos/Mi Juego (USA).nes", vec![1, 2, 3]);
        assert_eq!(rom.name, "Mi Juego (USA)");
        assert_eq!(rom.path, None);
    }

    #[test]
    fn read_reports_missing_file() {
        let err = RomFile::read(Path::new("no-existe-12345.nes")).unwrap_err();
        assert!(err.contains("no-existe-12345.nes"), "{err}");
    }

    #[test]
    fn read_returns_file_contents() {
        let path = std::env::temp_dir().join("replica-nes-test-read.nes");
        std::fs::write(&path, b"NES\x1a").unwrap();
        let rom = RomFile::read(&path).unwrap();
        std::fs::remove_file(&path).unwrap();

        assert_eq!(rom.name, "replica-nes-test-read");
        assert_eq!(rom.data, b"NES\x1a");
        assert_eq!(rom.path.as_deref(), Some(path.as_path()));
    }
}
