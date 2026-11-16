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
    /// Identificador estable de la ROM (nombre y huella del contenido), con el que se
    /// nombran sus partidas y estados guardados.
    pub id: String,
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
        let name = display_name(path);
        Ok(Self {
            id: rom_id(&name, &data),
            name,
            data,
            path: Some(path.to_path_buf()),
        })
    }

    /// Crea una ROM a partir de bytes ya en memoria (archivo soltado en la web, demo).
    pub fn from_bytes(file_name: &str, data: Vec<u8>) -> Self {
        let name = display_name(Path::new(file_name));
        Self {
            id: rom_id(&name, &data),
            name,
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

/// Identificador de una ROM: su nombre (solo caracteres válidos en un nombre de archivo)
/// más una huella del contenido, para que dos ROMs distintas con el mismo nombre no
/// compartan partida.
fn rom_id(name: &str, data: &[u8]) -> String {
    let safe: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, ' ' | '-' | '_' | '(' | ')') {
                c
            } else {
                '_'
            }
        })
        .collect();
    // FNV-1a de 32 bits: suficiente para distinguir ROMs, sin dependencias.
    let hash = data.iter().fold(0x811c_9dc5_u32, |hash, &byte| {
        (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193)
    });
    format!("{safe}-{hash:08x}")
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

/// Niveles de subcarpetas que se recorren al buscar juegos.
const LIBRARY_DEPTH: usize = 3;
/// Tope de juegos listados, para que una carpeta enorme no bloquee la app.
const LIBRARY_MAX: usize = 2000;

/// Un juego de la biblioteca: un archivo `.nes` de la carpeta de juegos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryEntry {
    /// Nombre para mostrar: el del archivo sin extensión.
    pub name: String,
    pub path: PathBuf,
    /// Tamaño del archivo en bytes.
    pub size: u64,
}

impl LibraryEntry {
    /// Tamaño en kilobytes, como se muestra en la lista.
    pub const fn size_kb(&self) -> u64 {
        self.size.div_ceil(1024)
    }
}

/// Busca los archivos `.nes` de `dir` y sus subcarpetas, ordenados por nombre.
///
/// Las carpetas o archivos que no se pueden leer se omiten: una biblioteca incompleta
/// es más útil que un error.
pub fn scan_library(dir: &Path) -> Vec<LibraryEntry> {
    let mut entries = Vec::new();
    scan_dir(dir, LIBRARY_DEPTH, &mut entries);
    entries.sort_by_cached_key(|entry| entry.name.to_lowercase());
    entries
}

fn scan_dir(dir: &Path, depth: usize, entries: &mut Vec<LibraryEntry>) {
    let Ok(read_dir) = std::fs::read_dir(dir) else {
        return;
    };
    for item in read_dir.flatten() {
        if entries.len() >= LIBRARY_MAX {
            return;
        }
        let path = item.path();
        let Ok(metadata) = item.metadata() else {
            continue;
        };
        if metadata.is_dir() {
            if depth > 0 {
                scan_dir(&path, depth - 1, entries);
            }
        } else if is_rom_path(&path) && metadata.len() <= MAX_ROM_BYTES {
            entries.push(LibraryEntry {
                name: display_name(&path),
                size: metadata.len(),
                path,
            });
        }
    }
}

/// Si el archivo tiene extensión `.nes`, sin distinguir mayúsculas.
fn is_rom_path(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("nes"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_lists_roms_sorted_and_ignores_other_files() {
        let dir = std::env::temp_dir().join("replica-nes-test-library");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("zeta.nes"), [0u8; 2000]).unwrap();
        std::fs::write(dir.join("Alfa.NES"), [0u8; 10]).unwrap();
        std::fs::write(dir.join("notas.txt"), b"no es un juego").unwrap();
        std::fs::write(dir.join("sub").join("beta.nes"), [0u8; 1024]).unwrap();

        let library = scan_library(&dir);
        std::fs::remove_dir_all(&dir).unwrap();

        let names: Vec<&str> = library.iter().map(|entry| entry.name.as_str()).collect();
        assert_eq!(names, ["Alfa", "beta", "zeta"]);
        assert_eq!(library[1].path, dir.join("sub").join("beta.nes"));
        let sizes: Vec<u64> = library.iter().map(LibraryEntry::size_kb).collect();
        assert_eq!(sizes, [1, 1, 2]);
    }

    #[test]
    fn library_of_a_missing_folder_is_empty() {
        assert!(scan_library(Path::new("no-existe-esta-carpeta-12345")).is_empty());
    }

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
    fn id_is_filename_safe_and_depends_on_contents() {
        let a = RomFile::from_bytes("Mi*Juego?.nes", vec![1, 2, 3]);
        let b = RomFile::from_bytes("Mi*Juego?.nes", vec![1, 2, 4]);
        assert!(a.id.starts_with("Mi_Juego_-"), "{}", a.id);
        assert!(!a.id.contains(['*', '?']));
        assert_ne!(a.id, b.id);
        assert_eq!(a.id, RomFile::from_bytes("Mi*Juego?.nes", vec![1, 2, 3]).id);
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
