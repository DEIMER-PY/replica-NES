# Cómo contribuir

La guía completa está en [docs/GUIA-DESARROLLO.md](docs/GUIA-DESARROLLO.md). Resumen:

1. Parte siempre de `develop`: `git checkout develop && git pull`.
2. Crea una rama `feature/<nombre>` (o `fix/<nombre>`, `docs/<nombre>`, `chore/<nombre>`).
3. Haz commits pequeños con el formato `<emoji> <tipo>(<ámbito>): <descripción>` en inglés.
4. Antes de subir, comprueba que pasa todo:

   ```sh
   cargo fmt --all --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   ```

5. Abre un pull request contra `develop`. La CI debe quedar en verde.
6. Actualiza la documentación y el `CHANGELOG.md` en el mismo PR.

No subas ROMs ni ningún otro material con derechos de autor.
