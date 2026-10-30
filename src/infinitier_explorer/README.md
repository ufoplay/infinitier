# Infinitier Explorer

A resource browser for Infinity Engine games: a [Slint](https://slint.dev) UI over the
`infinitier_core` data layer (game-data indexing, every resource importer, the sound and movie
decoders), built entirely on the [slint-themekit](https://github.com/ufoscout/slint-themekit)
component library (no `std-widgets`), so one `Theme` value styles everything.

## Build prerequisites

The same as the keeper's (see [its README](../infinitier_keeper/README.md#build-prerequisites)):

- **slint-themekit**: a path dependency on a sibling checkout for now, at
  `../../../github_ufoscout/slint-themekit` relative to the workspace root.
- **fontconfig** dev package on Linux (`sudo apt install libfontconfig1-dev`): Slint's text stack
  links it at build time.

## Run

```sh
cargo run -p infinitier_explorer
just run_explorer     # with the games of config_test/infinitier_explorer.toml
```

- `--config <file>` — the settings file, default `infinitier_explorer.toml` in the folder the
  explorer is started from. Created on the first change if it doesn't exist.
- `--log` — env_logger filter, default `infinitier=debug,warn`.

## Configuration

The explorer keeps its games in its settings file, the same format as the keeper's
`infinitier_keeper.toml` (see [its README](../infinitier_keeper/README.md#configuration)):

```toml
[settings]
# Index into `games` (from 0) of the game loaded at startup.
active = 0
theme = "dark"

[[games]]
game_path = "/games/Baldur's Gate II - Enhanced Edition"
save_path = "/home/me/Documents/Baldur's Gate II - Enhanced Edition"

[[games]]
game_path = "/games/Planescape Torment"
game_type = "Pst"
```

- **`game_path`** — the install folder, the one containing `CHITIN.KEY`.
- **`save_path`** (optional) — the folder containing the `save/` directory, when it isn't inside
  the install (the Enhanced Editions keep it under the user's Documents). Its files are merged
  over the install's, so its overrides and saved games show in the tree too.
- **`game_type`** (optional) — forces the game type instead of detecting it from the install.

Click **Settings** in the header to add or remove games: select one and close the dialog (or
double-click it) to load it in place of the current one. The explorer starts with the last game
loaded; with none configured it starts empty, pointing at Settings.

## What it does

- The **resource tree**: one group per file extension, `(O)` for override files, closed at first.
  Click a group to open it, a resource to view it; the arrow keys move through the tree (Left /
  Right close and open groups). The panel is resizable with the splitter.
- The **viewers**, each over an info bar (size, format, origin…):
  - images (BMP, PVRZ, MOS, PNG…), fitted to the area;
  - **BAM** animations: Play / Pause, cycle picker, frame slider;
  - **TIS** tilesets: tiles per row, tile grid, the too-large warning;
  - **FNT**: the struct table and a selectable hex dump of the body;
  - **TTF**: the font's name and metadata, and an editable sample at five sizes, drawn in the font;
  - **BCS** scripts, decompiled to BAF with syntax highlighting (light and dark palettes);
  - **sounds** (ACM, WAV, WAVC) and **movies** (MVE, BIK, WBM), streamed with their players;
  - a message for formats without a viewer yet ("ARE Viewer"…), unknown types and errors.
- The **header bar**: the loaded game, the **Dark / Light** theme toggle (remembered in the
  settings file) and **Settings**; at the bottom, the status line.
- A game loads on a background thread, behind a "Loading…" overlay; a game that fails to load
  reopens Settings with the error and leaves the loaded one in place.

The BAF view isn't selectable text: a **Copy** button copies the script, and long lines scroll
sideways instead of wrapping. The TTF sample sentence is kept from one font to the next.
`PORT_PLAN.md` records how this UI was built.

## Verifying the UI

Like the keeper, it can expose Slint's embedded MCP server — see
[`../../docs/slint_mcp.md`](../../docs/slint_mcp.md):

```sh
SLINT_EMIT_DEBUG_INFO=1 SLINT_MCP_PORT=8080 SLINT_BACKEND=headless TMPDIR=/some/scratch/dir \
cargo run -p infinitier_explorer --features mcp -- --config config_test/infinitier_explorer.toml
```

## Layout

Organised like the keeper:

- `ui/app.slint` — the window: composes the parts below, switches the themekit's `Theme` between
  its Light and Dark presets, and holds the resource panel's width. `ui/themes/` — the explorer's
  own layout values (`metrics.slint`) and data colours (`colors.slint`: the BAF palettes, the
  TIS grid); the only `.slint` files with literal design values (`tests/ui_tokens.rs` enforces it,
  and that nothing imports `std-widgets`).
- `ui/components/` — the explorer's own widgets on themekit components (info bar, fitted image,
  monospace text, hex dump, player transport); `ui/shell/` — the header bar, the resource panel
  and the status line; `ui/viewers/` — one file per viewer and their host; `ui/dialogs/` — the
  Settings dialog and the loading overlay. Each part reads and calls its `*Adapter` global.
- `build.rs` — compiles the `.slint` via `slint-build` and bakes in the git version.
- `src/main.rs` — clap args and startup; `src/app.rs` — the state; `src/loading.rs` — background
  game loading; `src/tree.rs` — the resource tree and its Slint model; `src/viewers/` — one
  module per viewer (state and logic); `src/view/` — fills the adapters, per UI part;
  `src/callbacks/` — handles their callbacks, per UI part; `src/ticker.rs` — the timer that
  advances a playing viewer; `src/fonts.rs` — runtime fonts (the system's monospace family, the
  TTF resources). The settings file and the Settings dialog's state are shared with the keeper,
  in the `infinitier_gui_common` crate (`src/infinitier_gui_common/`).
