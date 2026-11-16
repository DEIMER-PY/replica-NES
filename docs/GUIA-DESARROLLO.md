# Guía de desarrollo

## Entorno

| Herramienta | Para qué | Instalación |
|---|---|---|
| Rust estable | Compilar | `winget install Rustlang.Rustup` o <https://rustup.rs> |
| C++ Build Tools (Windows) | Enlazador MSVC | Visual Studio Installer → "Desarrollo para el escritorio con C++" |
| `trunk` + target wasm | Versión web | `cargo install trunk` y `rustup target add wasm32-unknown-unknown` |
| `gh` | Pull requests desde la terminal | `winget install GitHub.cli` |

En Linux hacen falta además las librerías de sistema que usa la CI (ver `.github/workflows/ci.yml`).

## Comandos habituales

```sh
cargo run -p nes-app                 # ejecutar la app (desarrollo)
cargo run -p nes-app --release       # ejecutar optimizada
cargo test --workspace               # tests
cargo fmt --all                      # formatear
cargo clippy --workspace --all-targets -- -D warnings   # lints
```

El core de emulación (`tetanes-core` y `nes-core`) se compila optimizado incluso en modo desarrollo (ver `[profile.dev.package.*]` en `Cargo.toml`), porque sin optimizar no alcanza la velocidad real. El resto de dependencias no, para que compilar sea rápido.

## Flujo de ramas

```
main      ← solo releases, siempre estable y etiquetada (v0.1.0, ...)
develop   ← rama por defecto; integración de todo el trabajo
feature/* ← una rama por funcionalidad, sale de develop y vuelve a develop
```

1. `git checkout develop && git pull`
2. `git checkout -b feature/<nombre>`
3. Commits pequeños y atómicos.
4. `git push -u origin feature/<nombre>` y `gh pr create --base develop`.
5. Con la CI en verde, merge a `develop` (merge commit, se conserva el historial de la feature).
6. Para publicar: PR de `develop` a `main` y tag `vX.Y.Z`.

Prefijos de rama: `feature/`, `fix/`, `docs/`, `chore/`, `refactor/`.

## Convención de commits

Formato (en inglés, imperativo, sin punto final, máximo ~72 caracteres):

```
<emoji> <tipo>(<ámbito>): <descripción>
```

| Emoji | Tipo | Uso |
|---|---|---|
| ✨ | `feat` | Funcionalidad nueva |
| 🐛 | `fix` | Corrección de un error |
| 📝 | `docs` | Solo documentación |
| ♻️ | `refactor` | Cambio interno sin alterar comportamiento |
| ⚡ | `perf` | Mejora de rendimiento |
| ✅ | `test` | Añadir o corregir tests |
| 🔧 | `chore` | Configuración, dependencias, mantenimiento |
| 👷 | `ci` | Integración continua |
| 💄 | `style` | Interfaz o formato, sin cambio de lógica |
| 🎉 | `chore` | Commit inicial |
| 🔖 | `chore(release)` | Publicación de versión |

Ámbitos habituales: `core`, `app`, `video`, `audio`, `input`, `ui`, `config`, `web`, `docs`, `ci`.

Ejemplos:

```
✨ feat(core): add Emulator wrapper around tetanes-core
🐛 fix(audio): avoid buffer underrun when window is unfocused
📝 docs(architecture): describe the frame loop
```

## Documentación

Cada PR actualiza la documentación que toca: `docs/USO.md` si cambia algo visible para el usuario, `docs/ARQUITECTURA.md` si cambia la estructura, `docs/ROADMAP.md` al completar una funcionalidad y `CHANGELOG.md` siempre.

## ROMs para probar

No se versionan ROMs. Para pruebas manuales usa homebrew de libre distribución o volcados de tus propios cartuchos, guardados fuera del repositorio o en una carpeta ignorada. Los tests automáticos generan en memoria una ROM mínima propia.
