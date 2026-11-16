# Uso

> La app está en construcción. Este documento se completa con cada funcionalidad.

## Ejecutar

```sh
cargo run -p nes-app --release
```

Se abre una ventana con la **ROM de demostración** incluida en el proyecto: una pantalla azul que confirma que la emulación y el vídeo funcionan.

La imagen se escala en múltiplos enteros (×1, ×2, ×3...) y se centra al redimensionar la ventana.

## Solución de problemas

**La app se cierra al abrir o no muestra ventana.** Suele ser el driver de la gráfica. En Windows la app usa DirectX 12 por defecto (Vulkan falla con algunos drivers de gráficas integradas Intel). Se puede forzar otro backend con la variable de entorno `WGPU_BACKEND`:

```powershell
$env:WGPU_BACKEND = "gl"      # o "dx12", "vulkan"
cargo run -p nes-app --release
```

## ROMs

El emulador abre archivos `.nes` (formato iNES / NES 2.0). El proyecto no incluye ROMs: usa volcados de tus propios cartuchos o juegos homebrew de libre distribución.
