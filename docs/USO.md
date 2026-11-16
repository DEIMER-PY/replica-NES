# Uso

> La app está en construcción. Este documento se completa con cada funcionalidad.

## Ejecutar

```sh
cargo run -p nes-app --release
```

Sin argumentos se abre con la **ROM de demostración** incluida en el proyecto: una pantalla azul que confirma que la emulación y el vídeo funcionan.

## Cargar una ROM

| Forma | Cómo |
|---|---|
| Menú | **Archivo → Abrir ROM…** (`Ctrl+O`) |
| Arrastrar y soltar | Suelta un archivo `.nes` sobre la ventana |
| Línea de comandos | `cargo run -p nes-app --release -- ruta/al/juego.nes` |
| Recientes | **Archivo → Recientes** guarda las últimas 8 ROMs abiertas |

El título de la ventana muestra el nombre de la ROM cargada. **Archivo → Cerrar ROM** extrae el cartucho y **Archivo → Cargar demo** vuelve a la demostración.

Si el archivo no es una ROM válida o usa un mapper no soportado, aparece una ventana de error con el motivo.

Para generar la demo como archivo y probar la carga desde disco:

```sh
cargo run -p nes-core --example dump_demo -- demo.nes
```

## Imagen

La imagen se centra y se escala al redimensionar la ventana. Por defecto solo en múltiplos enteros (×1, ×2, ×3...), para que todos los píxeles midan lo mismo; se cambia en **Ver → Escalado entero**.

## Configuración

La lista de recientes y las opciones de imagen se guardan automáticamente y se recuperan al abrir la app. En Windows el archivo está en `%APPDATA%\replica-NES\data\app.ron`.

## Solución de problemas

**La app se cierra al abrir o no muestra ventana.** Suele ser el driver de la gráfica. En Windows la app usa DirectX 12 por defecto (Vulkan falla con algunos drivers de gráficas integradas Intel). Se puede forzar otro backend con la variable de entorno `WGPU_BACKEND`:

```powershell
$env:WGPU_BACKEND = "gl"      # o "dx12", "vulkan"
cargo run -p nes-app --release
```

## ROMs

El emulador abre archivos `.nes` (formato iNES / NES 2.0). El proyecto no incluye ROMs: usa volcados de tus propios cartuchos o juegos homebrew de libre distribución.
