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
cargo run -p infinitier_keeper_slint -- --config config_test/infinitier_keeper.toml
```

- `--config` — the settings file (default `infinitier_keeper.toml` in the current folder). It uses
  the same TOML format as the egui keeper's, so both keepers can share one.
- `--log` — env_logger filter, default `infinitier=debug,warn`.

The keeper starts with no game: it loads the settings file's active game (behind a "Loading…"
overlay with a spinner), and **Settings** adds games and picks another. **Load** then opens a save.

## What works

This is a **complete** reimplementation of the egui keeper's GUI. All **15 character tabs** are
ported (Abilities, Levels & Kits, Characteristics, Appearance, Inventory, Spells, Feats,
Memorization, Proficiencies, Resistances, Effects, Local/Global Variables, Journal Entries,
Miscellaneous), for both the AD&D engines and IWD2 (d20). Editable fields clamp to the engine's
caps on commit exactly like the egui keeper.

Also ported: the **Settings** dialog (games list with autodetected types, add/remove, game
switching on a background thread), the **Dark/Light** theme (saved in the settings file), the
floating **Item Browser** and **Spell Browser** (filters, icon + description, add items/spells
incl. the IWD2 spellbook choice, double-click a slot or spell to reveal it), the **Load** dialog (newest first, with
screenshot + portrait preview and Delete) + **multi-save tab strip** (open several saves at once,
switch/close), party **portraits**, and **Save** (copies the save folder into a new
`… (Edited NNNN)` folder and re-exports the edited GAM into it, never touching the original).

One egui nicety is still missing (see `PORT_STATUS.md`): a wrapping character tab strip.

## Verifying the UI

This crate can expose Slint's embedded MCP server for automated / AI-driven UI inspection — see
[`../../docs/slint_mcp.md`](../../docs/slint_mcp.md). In short:

```sh
SLINT_EMIT_DEBUG_INFO=1 SLINT_MCP_PORT=8080 SLINT_BACKEND=headless \
cargo run -p infinitier_keeper_slint --features mcp -- --config config_test/infinitier_keeper.toml
```

## Layout

The UI is split like Slint's demo apps, and mirrors the egui keeper's modules:

- `ui/app.slint` — the window: composes the parts below (plain `std-widgets`, coloured from their
  `Palette` so the dark / light theme applies); `ui/theme.slint` — the few extra colours.
- `ui/components/` — reusable widgets (list rows, chip strip, modal frame, floating window, field
  card); `ui/shell/` — header, save tabs, party list, character panel; `ui/tabs/` — the character
  tabs' pages; `ui/browser/` — the Item / Spell Browser windows; `ui/dialogs/` — the modal
  dialogs. Each part reads and calls its `*Adapter` global.
- `build.rs` — compiles the `.slint` via `slint-build`.
- `src/main.rs` — clap args and startup; `src/app.rs` — the top-level state (no game / game /
  open saves) and background game loading; `src/view/` — fills the adapters, per UI part;
  `src/callbacks/` — handles their callbacks, per UI part; `src/tabs/` — one module per character
  tab; `src/components/` — the shared field logic; `src/dialogs/` — the dialogs' state;
  `src/browser/` — the browsers' indexes and add logic; `src/settings.rs` — the settings file.
