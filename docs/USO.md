# Uso

## Ejecutar

Con el ejecutable (`nes-app.exe` en Windows): doble clic. No necesita instalación ni terminal.

Desde el código fuente:

```sh
cargo run -p nes-app --release
```

Sin argumentos se abre con la **ROM de demostración** incluida en el proyecto: una pantalla azul que pasa a rojo, con un tono, mientras se mantiene el botón A (`X`). Confirma que la emulación, el vídeo, los controles y el sonido funcionan.

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

## Controles

### Teclado

| Botón NES | Jugador 1 | Jugador 2 |
|---|---|---|
| Arriba / Abajo / Izquierda / Derecha | Flechas | `I` / `K` / `J` / `L` |
| A | `X` | `H` |
| B | `Z` | `G` |
| Select | `Retroceso` | `T` |
| Start | `Enter` | `Y` |

Son las teclas por defecto. Se cambian en **Archivo → Ajustes… → Controles** (`Ctrl+,`): se hace clic en la tecla de la tabla y se pulsa la nueva. `Escape` cancela.

- Si la tecla nueva ya estaba en otro botón, los dos botones intercambian sus teclas.
- Las teclas que usa la app como atajo (`P`, `M`, `F`, `F5`, `F8`, `F11`, `1` a `4`) no se pueden asignar.
- **Restaurar teclas por defecto** vuelve a la tabla de arriba.

La misma pestaña muestra los mandos conectados.

El teclado no llega al juego mientras hay un menú abierto o se mantiene `Ctrl` o `Alt`, para que los atajos de la app no muevan al personaje.

### Mandos

Los mandos USB o Bluetooth se detectan solos, también si se conectan con la app abierta. El primero controla al jugador 1 y el segundo al jugador 2; teclado y mando funcionan a la vez.

| Botón NES | Mando |
|---|---|
| Dirección | Cruceta o stick izquierdo |
| A | Botón inferior (A en Xbox, ✕ en PlayStation) |
| B | Botón izquierdo (X en Xbox, □ en PlayStation) |
| Select / Start | Select / Start |

Son los botones por defecto. Se cambian en **Archivo → Ajustes… → Controles**, en la tabla **Mandos**: se hace clic en el botón de la tabla y se pulsa el nuevo en el mando de ese jugador. `Escape` cancela.

- Cada jugador tiene su propia tabla, y solo se puede editar con su mando conectado.
- Si el botón nuevo ya estaba asignado, los dos intercambian su sitio.
- El stick izquierdo siempre funciona como dirección, además de los botones asignados.
- **Restaurar botones por defecto** vuelve a la tabla de arriba.

En los **mandos genéricos** el sistema a veces no sabe qué botón es cada uno, y pueden salir cambiados de sitio. Se arregla reasignándolos; los botones sin posición conocida aparecen en la tabla como `Código …`.

Con la ROM de demostración se puede comprobar: al mantener el botón A del jugador 1 la pantalla pasa de azul a rojo.

## Emulación

Todo está en el menú **Emulación** y tiene atajo:

| Acción | Atajo | Qué hace |
|---|---|---|
| Pausar / Reanudar | `P` | Detiene la consola; se indica con **PAUSA** en pantalla |
| Reiniciar | `Ctrl+R` | Botón RESET de la consola: conserva la memoria |
| Apagar y encender | `Ctrl+Shift+R` | Reinicio completo: borra la memoria (no la partida con batería) |
| Avance rápido | mantener `F` | Emula a ×4, sin sonido, mientras se mantiene la tecla |
| Guardar estado | `F5` | Guarda el instante exacto del juego en la ranura seleccionada |
| Cargar estado | `F8` | Vuelve al instante guardado en la ranura seleccionada |
| Elegir ranura | `1` a `4` | Cuatro ranuras independientes por juego |

### Partidas y estados guardados

Hay dos formas de guardar, y son independientes:

- **Partida del juego (batería).** Los juegos que guardaban en el cartucho lo hacen igual aquí. La app escribe esa memoria en disco sola: cada 30 segundos, al cambiar o cerrar la ROM y al salir.
- **Estados (save states).** Una foto completa de la consola que se guarda y se carga cuando el usuario quiere, en cualquier juego.

Ambos se guardan en la carpeta de datos de la app, no junto a la ROM. En Windows:

```
%APPDATA%\replica-NES\data\saves\<juego>.sav
%APPDATA%\replica-NES\data\states\<juego>.<ranura>.state
```

El nombre incluye una huella del contenido de la ROM, así que dos versiones distintas del mismo juego no se pisan la partida.

## Sonido

El sonido sale por el dispositivo de audio por defecto del sistema. En el menú **Audio**:

- **Silenciar** (tecla `M`).
- **Volumen**, de 0 a 100 %.

Ambos ajustes se recuerdan. Si no hay ningún dispositivo de audio, la app funciona igual pero en silencio, y el menú **Audio** indica el motivo.

Con la ROM de demostración se puede comprobar: suena un tono mientras se mantiene el botón A del jugador 1.

## Imagen

La imagen se centra y se escala al redimensionar la ventana. Las opciones están en el menú **Ver** y en **Ajustes… → Vídeo**:

| Opción | Qué hace |
|---|---|
| Escalado entero | Escala solo ×1, ×2, ×3..., para que todos los píxeles midan lo mismo. Activado por defecto |
| Proporción de televisor (8:7) | Ensancha la imagen como se veía en un televisor de la época, en lugar de píxeles cuadrados |
| Pantalla completa | `F11` |

## Configuración

La lista de recientes, las teclas, los botones de mando y las opciones de imagen y sonido se guardan automáticamente y se recuperan al abrir la app. En Windows el archivo está en `%APPDATA%\replica-NES\data\app.ron`.

## Solución de problemas

**La app se cierra al abrir o no muestra ventana.** Suele ser el driver de la gráfica. En Windows la app usa DirectX 12 por defecto (Vulkan falla con algunos drivers de gráficas integradas Intel). Se puede forzar otro backend con la variable de entorno `WGPU_BACKEND`:

```powershell
$env:WGPU_BACKEND = "gl"      # o "dx12", "vulkan"
cargo run -p nes-app --release
```

## ROMs

El emulador abre archivos `.nes` (formato iNES / NES 2.0). El proyecto no incluye ROMs: usa volcados de tus propios cartuchos o juegos homebrew de libre distribución.
