# Infinitier Keeper (Slint edition)

A [Slint](https://slint.dev) reimplementation of `infinitier_keeper`'s GUI. It reuses the entire
data layer from `infinitier_core` (game-data indexing, save discovery, GAM/CRE parse + export,
engine caps) and swaps only the *view*: vanilla `std-widgets` Slint in place of egui.

## Build prerequisite (Linux)

Slint's text stack links **fontconfig** at build time. Install the dev package once:

```sh
sudo apt install libfontconfig1-dev      # Debian/Ubuntu
sudo dnf install fontconfig-devel        # Fedora
```

The runtime `libfontconfig.so.1` on its own is not sufficient — the build needs the `.pc` +
`.so` dev symlink for the linker. (No extra GL dev library is required: this crate uses Slint's
software renderer + winit backend.)

Windows and macOS need no extra system package.

## Run

```sh
cargo run -p infinitier_keeper_slint -- \
    --savegame 0 \
    --game-path "/path/to/game"          # folder containing CHITIN.KEY
```

- `--savegame` — a numeric index (alphabetical, from 0) or the save folder name.
- `--game-path` — one or more comma-separated game folders.
- `--log` — env_logger filter, default `infinitier=debug,warn`.

## What works

This is a **complete** reimplementation of the egui keeper's GUI. All **15 character tabs** are
ported (Abilities, Levels & Kits, Characteristics, Appearance, Inventory, Spells, Feats,
Memorization, Proficiencies, Resistances, Effects, Local/Global Variables, Journal Entries,
Miscellaneous), for both the AD&D engines and IWD2 (d20). Editable fields clamp to the engine's
caps on commit exactly like the egui keeper.

Also ported: the **Item Browser** and **Spell Browser** (add items/spells), the **Load** dialog +
**multi-save tab strip** (open several saves at once, switch/close), party **portraits**, and
**Save (copy)** (copies the save folder and re-exports the edited GAM into the copy, never touching
the original).

A few egui niceties are intentionally left out (see `PORT_STATUS.md`): IWD2 spell-add via the
list-2DA path, double-click "reveal in browser", the browsers' icon/description detail panels, and
the Load dialog's screenshot/portrait preview + delete.

## Verifying the UI

This crate can expose Slint's embedded MCP server for automated / AI-driven UI inspection — see
[`../../docs/slint_mcp.md`](../../docs/slint_mcp.md). In short:

```sh
SLINT_EMIT_DEBUG_INFO=1 SLINT_MCP_PORT=8080 SLINT_BACKEND=headless \
cargo run -p infinitier_keeper_slint --features mcp -- --savegame 0 --game-path "…"
```

## Layout

- `ui/app.slint` — the declarative view (kept deliberately plain: `std-widgets` only, no theming).
- `build.rs` — compiles the `.slint` via `slint-build`.
- `src/main.rs` — the bridge: clap args, the core load pipeline, the shared `Model`, and the
  Slint callbacks (`select-party`, `commit-field`, `save`).
