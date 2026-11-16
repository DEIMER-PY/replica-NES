//! Configuración persistente de la app.
//!
//! Se guarda con el almacenamiento de `eframe`: un archivo en la carpeta de datos
//! del usuario en escritorio y `localStorage` en la web.

use serde::{Deserialize, Serialize};

use crate::rom::RecentRoms;

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
}

impl Default for Config {
    fn default() -> Self {
        Self {
            recent_roms: RecentRoms::default(),
            integer_scaling: true,
        }
    }
}

impl Config {
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
