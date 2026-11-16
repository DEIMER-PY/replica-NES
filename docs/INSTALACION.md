# Instalación

Hay dos formas de tener replica-NES: descargar el **ejecutable** ya compilado (Windows y Linux) o **compilarlo** desde el código fuente (Windows, Linux y macOS). También existe una [versión web](WEB.md) que funciona en el navegador.

| Sistema | Ejecutable descargable | Compilar desde el código |
|---|---|---|
| Windows 10 / 11 (64 bits) | Sí | Sí |
| Linux (64 bits) | Sí | Sí |
| macOS | No | Sí |

> **Estado de las pruebas.** La app se ha ejecutado y probado en Windows 11. En Linux la integración continua la compila y pasa los tests, pero no se ha usado con ventana. En macOS no se ha compilado ni probado: los pasos son los estándar de Rust y de las librerías que usa el proyecto.

## Requisitos mínimos

La NES es una consola de 1983 y emularla exige poco. Estos requisitos son orientativos: salen de las librerías que usa la app y de medir en el equipo de desarrollo, no de probar en equipos antiguos.

| | Mínimo |
|---|---|
| Sistema | Windows 10 u 11, Linux con X11 o Wayland, o macOS 11 o posterior; siempre de 64 bits |
| Procesador | Cualquiera de dos núcleos de los últimos 10 a 12 años |
| Gráfica | Integrada, compatible con DirectX 12, Vulkan, Metal u OpenGL 3.3 |
| Memoria | 4 GB en el equipo (la app usa unos 350 MB) |
| Disco | 20 MB para el ejecutable; unos 4 GB si se compila desde el código |
| Sonido | Opcional: sin dispositivo de audio la app funciona en silencio |
| Mando | Opcional: USB o Bluetooth; se puede jugar solo con teclado |

Medido en un portátil con Intel Core i3-10110U y gráfica integrada Intel UHD: emular a velocidad real ocupa alrededor de un 30 % de un núcleo, y la app usa unos 330 MB de memoria.

## Opción A: ejecutable

1. Abre la página de [Releases](https://github.com/DEIMER-PY/replica-NES/releases) del repositorio. El repositorio es privado: hay que iniciar sesión en GitHub con una cuenta que tenga acceso.
2. Descarga el archivo de tu sistema de la versión más reciente.
3. Ábrelo. No hay instalador: es un único archivo que puedes guardar donde quieras.

### Windows

1. Descarga `replica-NES-vX.Y.Z-windows-x64.exe`.
2. Haz doble clic. La primera vez, Windows puede mostrar el aviso «Windows protegió su PC», porque el ejecutable no está firmado: pulsa **Más información → Ejecutar de todas formas**.
3. Para abrir un juego directamente, arrastra el archivo `.nes` sobre el ejecutable o sobre la ventana.

Si la ventana no aparece o se cierra al abrir, consulta [Solución de problemas](USO.md#solución-de-problemas).

### Linux

1. Descarga `replica-NES-vX.Y.Z-linux-x64`.
2. Instala las librerías que necesita. Suelen venir ya en un escritorio normal:

   ```sh
   # Debian, Ubuntu, Linux Mint
   sudo apt install libasound2 libudev1 libxkbcommon0 libgtk-3-0

   # Fedora
   sudo dnf install alsa-lib systemd-libs libxkbcommon gtk3

   # Arch, Manjaro
   sudo pacman -S alsa-lib systemd-libs libxkbcommon gtk3
   ```

3. Dale permiso de ejecución y ábrelo:

   ```sh
   chmod +x replica-NES-vX.Y.Z-linux-x64
   ./replica-NES-vX.Y.Z-linux-x64            # con la demo
   ./replica-NES-vX.Y.Z-linux-x64 juego.nes  # con un juego
   ```

El ejecutable se compila en Ubuntu reciente: en distribuciones antiguas puede fallar por la versión de `glibc`. En ese caso, compílalo (opción B).

## Opción B: compilar desde el código

Los pasos son los mismos en los tres sistemas; solo cambian las herramientas previas.

### 1. Herramientas previas

**Windows**

```powershell
winget install Rustlang.Rustup
winget install Git.Git
```

Además hace falta el enlazador de C++: en el [instalador de Visual Studio](https://visualstudio.microsoft.com/es/downloads/) (vale «Build Tools for Visual Studio»), marca **Desarrollo para el escritorio con C++**, que incluye el SDK de Windows. Abre una terminal nueva al terminar.

**Linux**

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Debian, Ubuntu, Linux Mint
sudo apt install git build-essential pkg-config libasound2-dev libudev-dev libxkbcommon-dev libwayland-dev libgtk-3-dev

# Fedora
sudo dnf install git gcc pkgconf-pkg-config alsa-lib-devel systemd-devel libxkbcommon-devel wayland-devel gtk3-devel

# Arch, Manjaro
sudo pacman -S git base-devel alsa-lib systemd-libs libxkbcommon wayland gtk3
```

**macOS**

```sh
xcode-select --install
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Descargar y compilar

```sh
git clone https://github.com/DEIMER-PY/replica-NES.git
cd replica-NES
cargo build -p nes-app --release
```

La primera compilación descarga las dependencias y tarda entre 10 y 20 minutos según el equipo. Hace falta Rust 1.95 o posterior; `rustup update` lo actualiza.

### 3. Ejecutar

El ejecutable queda en `target/release/nes-app` (`nes-app.exe` en Windows). Se puede copiar a cualquier carpeta.

```sh
./target/release/nes-app              # con la demo
./target/release/nes-app juego.nes    # con un juego
```

Para actualizar más adelante: `git pull` y de nuevo `cargo build -p nes-app --release`.

## Comprobar que funciona

Sin juego, la app abre una **ROM de demostración** propia: la pantalla se ve azul. Al mantener la tecla `X` (botón A) pasa a rojo y suena un tono. Con eso quedan comprobados el vídeo, los controles y el sonido. El resto de teclas y funciones está en [USO.md](USO.md).

## Dónde conseguir ROMs

El emulador abre archivos `.nes` (formato iNES / NES 2.0). **El proyecto no incluye ningún juego**, y este repositorio no enlaza a sitios de descarga de juegos comerciales: esos juegos tienen derechos de autor y descargarlos de internet es ilegal en la mayoría de países, aunque se tenga el cartucho original.

Formas legítimas de conseguir juegos:

- **Homebrew: juegos nuevos hechos para la NES.** Hay una comunidad activa que sigue publicando juegos, muchos gratuitos y otros de pago, que se descargan como archivo `.nes`:
  - [itch.io, juegos etiquetados NES](https://itch.io/games/tag-nes): filtra por los que ofrecen descarga de la ROM.
  - [NESdev Competition](https://www.nesdev.org/wiki/NESdev_Competition): juegos presentados al concurso anual de la comunidad NESdev.
  - Las páginas de los propios autores y editoriales de homebrew, que suelen vender la ROM junto al cartucho.
- **Volcados de tus propios cartuchos.** Con un lector de cartuchos (*dumper*) conectado por USB se copia el contenido de un cartucho propio a un archivo `.nes`. La legalidad de la copia privada depende de cada país.
- **ROMs de prueba.** La comunidad [NESdev](https://www.nesdev.org/wiki/Emulator_tests) mantiene programas de test de libre distribución, útiles para comprobar la precisión del emulador.
- **La demo del proyecto.** `cargo run -p nes-core --example dump_demo -- demo.nes` la escribe como archivo, para probar la carga desde disco.

Guarda las ROMs fuera de la carpeta del repositorio, o ten en cuenta que los `.nes` están en `.gitignore` para que no se suban por error.

## Dónde guarda sus datos la app

La configuración, las partidas con batería y los estados guardados no se escriben junto al ejecutable ni junto a la ROM:

| Sistema | Carpeta |
|---|---|
| Windows | `%APPDATA%\replica-NES\data` |
| Linux | `~/.local/share/replica-nes` |
| macOS | `~/Library/Application Support/replica-NES` |

Para desinstalar, borra el ejecutable y esa carpeta.
