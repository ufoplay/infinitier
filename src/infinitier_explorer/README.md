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
cargo run -p infinitier_explorer -- "/path/to/Baldur's Gate 2/"
just run_bg2          # or run_bg2ee, run_iwd2, … (just/run.just)
```

- `game_path` — comma-separated game folders; the first must hold `CHITIN.KEY`. Several folders
  merge into one view, later ones overriding earlier ones (an EE game plus its Documents folder).
- `--log` — env_logger filter, default `infinitier=debug,warn`.

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
- The **Dark / Light** theme toggle, and the status line.

The BAF view isn't selectable text: a **Copy** button copies the script, and long lines scroll
sideways instead of wrapping. The TTF sample sentence is kept from one font to the next.
`PORT_PLAN.md` records how this UI was built.

## Verifying the UI

Like the keeper, it can expose Slint's embedded MCP server — see
[`../../docs/slint_mcp.md`](../../docs/slint_mcp.md):

```sh
SLINT_EMIT_DEBUG_INFO=1 SLINT_MCP_PORT=8080 SLINT_BACKEND=headless TMPDIR=/some/scratch/dir \
cargo run -p infinitier_explorer --features mcp -- "/path/to/Baldur's Gate 2/"
```

## Layout

Organised like the keeper:

- `ui/app.slint` — the window: composes the parts below, switches the themekit's `Theme` between
  its Light and Dark presets, and holds the resource panel's width. `ui/themes/` — the explorer's
  own layout values (`metrics.slint`) and data colours (`colors.slint`: the BAF palettes, the
  TIS grid); the only `.slint` files with literal design values (`tests/ui_tokens.rs` enforces it,
  and that nothing imports `std-widgets`).
- `ui/components/` — the explorer's own widgets on themekit components (info bar, fitted image,
  monospace text, hex dump, player transport); `ui/shell/` — the resource panel and the status
  line; `ui/viewers/` — one file per viewer and their host. Each part reads and calls its
  `*Adapter` global.
- `build.rs` — compiles the `.slint` via `slint-build` and bakes in the git version.
- `src/main.rs` — clap args and startup; `src/app.rs` — the state; `src/tree.rs` — the resource
  tree and its Slint model; `src/viewers/` — one module per viewer (state and logic);
  `src/view/` — fills the adapters, per UI part; `src/callbacks/` — handles their callbacks, per
  UI part; `src/ticker.rs` — the timer that advances a playing viewer; `src/fonts.rs` — runtime
  fonts (the system's monospace family, the TTF resources).
