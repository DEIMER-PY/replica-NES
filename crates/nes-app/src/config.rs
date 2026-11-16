//! Configuración persistente de la app.
//!
//! Se guarda con el almacenamiento de `eframe`: un archivo en la carpeta de datos
//! del usuario en escritorio y `localStorage` en la web.

use serde::{Deserialize, Serialize};

use crate::{
    input::{KeyBindings, PadBindings},
    rom::RecentRoms,
    video::Scaling,
};

/// Clave bajo la que se guarda la configuración.
const STORAGE_KEY: &str = "config";

/// Ajustes que se conservan entre sesiones.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// ROMs abiertas recientemente.
    pub recent_roms: RecentRoms,
    /// Escalar la imagen solo en múltiplos enteros.
    pub integer_scaling: bool,
    /// Mostrar la imagen con la proporción de un televisor (píxeles 8:7).
    pub tv_aspect: bool,
    /// Teclas asignadas a los mandos.
    pub key_bindings: KeyBindings,
    /// Botones de mando asignados a los mandos de la NES.
    pub pad_bindings: PadBindings,
    /// Volumen del sonido, de 0 a 100.
    pub volume: u8,
    /// Sonido silenciado.
    pub muted: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            recent_roms: RecentRoms::default(),
            integer_scaling: true,
            tv_aspect: false,
            key_bindings: KeyBindings::default(),
            pad_bindings: PadBindings::default(),
            volume: 80,
            muted: false,
        }
    }
}

impl Config {
    /// Opciones de escalado de la imagen.
    pub const fn scaling(&self) -> Scaling {
        Scaling {
            integer: self.integer_scaling,
            tv_aspect: self.tv_aspect,
        }
    }

    /// Carga la configuración guardada, o la de por defecto si no hay o no es válida.
    pub fn load(storage: Option<&dyn eframe::Storage>) -> Self {
        storage
            .and_then(|storage| eframe::get_value(storage, STORAGE_KEY))
            .unwrap_or_default()
    }

    /// Guarda la configuración.
    pub fn save(&self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, STORAGE_KEY, self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_take_defaults() {
        // Una configuración antigua sin campos nuevos debe seguir cargando.
        let config: Config = ron::from_str("()").unwrap();
        assert_eq!(config, Config::default());
        assert!(config.integer_scaling);
    }

    #[test]
    fn round_trips_through_ron() {
        let mut config = Config {
            integer_scaling: false,
            ..Config::default()
        };
        config.recent_roms.push(std::path::Path::new("juego.nes"));

        let text = ron::to_string(&config).unwrap();
        assert_eq!(ron::from_str::<Config>(&text).unwrap(), config);
    }
}
