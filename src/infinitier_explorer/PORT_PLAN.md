# Infinitier Explorer (Slint): port plan

> **History.** The egui explorer this was ported from was removed on 2026-10-25, and the Slint
> port took over its name (`infinitier_explorer`, formerly `infinitier_explorer_slint`). This file
> records how the port was done; references to the egui explorer point at code that no longer
> exists.

A Slint version of `infinitier_explorer` (egui), with the same features. It is
built the way `infinitier_keeper` is:

- The UI uses only [slint-themekit](../../../../github_ufoscout/slint-themekit)
  components and `Theme` tokens, never `std-widgets`. Literal design values
  live only in `ui/themes/`. `tests/ui_tokens.rs` enforces both rules.
- `ui/` is split into small files: `components/`, `shell/`, `viewers/` and
  `themes/`, with barrel files. Each part reads and calls its own `*Adapter`
  global, and `app.slint` only composes the parts.
- On the Rust side, `viewers/` holds each viewer's state and logic, `view/`
  pushes state into the adapters, and `callbacks/` handles the adapters'
  callbacks.
- **No shared crate with the egui explorer.** Pure logic is duplicated here:
  the tree grouping, `ColumnLimits`, the BAF tokenizer, the audio and movie
  streaming engines, `build_version`. The egui explorer is not touched.
- Any widget the themekit is missing is added to the themekit, following its
  `CLAUDE.md` and `DESIGN_PRINCIPLES.md`, with a gallery demo in the same
  change.

## Feature inventory (egui explorer → Slint)

| egui explorer | Slint explorer |
|---|---|
| CLI: `game_path` (comma-separated, 1..n), `--log`; exit 1 if loading fails | same, duplicated `Args` |
| Window title `Infinitier Explorer {version} - {game:?}`, 1400×800 | same, `Metrics.window-*` |
| Left panel, resizable (default 260): theme toggle ("Light"/"Dark") + "Resources" heading + separator + tree | `ResourcePanel` + themekit `Splitter` (new) |
| Tree: one group per extension, `"{ext} ({n})"`, closed by default; leaves `NAME.ext`, `" (O)"` for override; selecting a leaf opens it | `ResourceTree` (duplicated grouping and tests) behind a custom `slint::Model`, rows are themekit `TreeItem`s (new) in a virtualized `ListView`; Up/Down/Left/Right keyboard navigation |
| Bottom bar: `Resource: {name} - Source: {origin:?}` / `No file selected` | `StatusBar` |
| Central viewer, cached per resource id | `ViewerHost` switching on `ViewerAdapter.kind` |
| "Select a resource from the panel on the left." / error / not found / `Unknown Viewer (type: 0x....)` / "XXX Viewer" stubs | `MessageViewer` |
| Info bar on every viewer: items separated by separators, size (`ByteSize` or `? B`), origin (`BIF: x`, `name: path`, `unhardcoded/x`, `Missing`) | `InfoBar` component, `viewers::info` (duplicated labels) |
| Image (BMP, PVRZ, …): fit to area, never upscaled, centred; info `W × H px`, size, format, description, origin | `ImageViewer` + `FittedImage` component |
| BAM: Play/Pause (when >1 frame), cycle combo, frame slider `/ max`, `(frame #n)`, wall-clock playback; info w×h, center, size, type, frames, cycles, origin; "No frames to display" | `BamViewer` with themekit `Select` and `Slider` (new), a `slint::Timer` ticker |
| TIS: "Tiles per row" slider (clamped by `ColumnLimits`), "Show grid", truncation warning; grid overlay; info | `TisViewer`, grid drawn as 1-border-width rectangles over the image |
| FNT: Attribute/Value/Offset/Size grid, note, hex dump of the first 256 body bytes; info | `FntViewer`; the hex dump is a read-only, selectable monospace `TextInput` |
| TTF: full name at 36 px in the font, metadata grid, sample text field, samples at 12/16/24/36/48 px, "(empty)"; info | `TtfViewer`; the font is registered at runtime through `slint::fontique_011` and used by family name |
| BCS: decompiled BAF, syntax-highlighted (context-aware triggers/actions), dark and light palettes, selectable; info CR blocks, BAF lines, size, origin | `BcsViewer`: duplicated tokenizer, one virtualized row per line of coloured spans in a monospace font, colours in `ui/themes/syntax.slint`; a **Copy** button copies the whole script (see deviations) |
| Sound (ACM/WAV/WAVC): streaming rodio player, progress, `pos / total`, Play/Pause + Stop, "no audio device" / decode error; info | `SoundViewer`, duplicated streaming engine, `ProgressBar`, a ticker polls the position |
| Movie (MVE/BIK/WBM): streaming decoder thread, A/V sync, transport, progress with time, video fitted to the area, "Movie Player" + size placeholder; info | `MovieViewer`, duplicated engine, frames pushed into an `Image` from the ticker |

### Deliberate deviations

- **BAF selection**: egui lets you select any part of the highlighted text.
  Slint's `TextInput` (the only selectable text) has one colour, and a whole
  script in one is not virtualized. The viewer keeps the colours and a fast
  virtualized list, and adds a **Copy** button for the script. Long lines
  scroll sideways instead of wrapping.
- **Inter font / X11 scale hack**: egui-specific. The Slint explorer uses the
  theme's font like the Slint keeper; Slint honours `SLINT_SCALE_FACTOR`.
- **TTF sample**: the sentence is kept from one font to the next (egui resets
  it per font). The same text field stays on screen between two TTFs, so one
  sentence for all keeps the field and the samples in step.
- **Small additions**: the tree moves with the arrow keys, and an import error
  is shown in the danger tone.

## Themekit additions

| Component | Why | API sketch |
|---|---|---|
| `Slider` | BAM frame, TIS columns | `value`, `minimum`, `maximum`, `step`, `enabled`; `changed(value)`; arrow keys; `slider` role with value, min, max, increment and decrement |
| `TreeItem` | resource tree | a `ListView` row: `text`, `level`, `expandable`, `expanded`, `selected`; `clicked()`, `toggled()`; expandable/expanded accessibility state |
| `Splitter` | resizable left panel | `orientation`; `dragged(delta)`; resize cursor, `StateLayer` hover |
| `ListView` sideways scroll | long BAF lines | scrolls sideways with a horizontal indicator when `content-width` is wider than the list, like `ScrollArea` |
| `VerticalDivider` | info bars, toolbars | a vertical hairline (`Divider` is horizontal, with a fixed height) |
| Icons | transport, tree, copy | `collapsed` (chevron-right), `play`, `pause`, `stop` (square), `copy` |

Monospace text stays in the app: there is no font-family literal that works
on every platform (Slint passes the family as a name, so `"monospace"` doesn't
resolve). The app asks fontique for the system's monospace family at startup
and publishes it in its `Fonts` global, and the themekit has no runtime code
to do the same (principle 10).

## Phases

0. **Themekit**: add the components and icons above, with gallery demos.
   Then run `cargo test --workspace` and `just check_code`, and look at the
   gallery in Light and Dark.
1. **Crate and shell**: `Cargo.toml`, `build.rs` (vergen-gix plus Slint with
   `@themekit`), the workspace member, `tests/ui_tokens.rs`, `main.rs`
   (arguments, loading, title), the resource tree with its model and tests,
   the splitter, the status bar and the theme toggle.
2. **Viewer framework**: `ViewerAdapter.kind`, `ViewerHost`, `MessageViewer`
   (none, stub, unknown, error), `InfoBar`, the per-resource dispatch
   (duplicated `match`), the ticker.
3. **Image, BAM, TIS** viewers (plus the `ColumnLimits` tests).
4. **FNT, TTF** viewers (hex dump test, runtime font registration).
5. **BCS** viewer (tokenizer tests, per-line spans, copy).
6. **Sound and movie** viewers.
7. **Finish**: `just` recipes, readme, MCP verification on BG2, BG2EE and
   IWD2 in Light and Dark, project `CLAUDE.md` and docs, memory. Nothing is
   committed.

## Status (2026-10-25): complete

Every phase landed:

0. **Themekit**: `Slider`, `TreeItem`, `Splitter`, `VerticalDivider`, `ListView`'s sideways
   scroll and the icons, each with a gallery demo. `cargo test --workspace` and `just check_code`
   pass, and the gallery was checked in Light and Dark.
1. **Shell**: the crate, the resource tree (a custom `slint::Model`, so opening a 17,000-row group
   adds rows instead of rebuilding the list), the splitter, the status line and the theme toggle.
2. **Viewer framework**: `ViewerHost`, the info bar, the dispatch, and the ticker that runs only
   while something plays.
3. to 6. **Every viewer**: images, BAM, TIS, FNT, TTF, BCS, sound and movie.
7. **Docs**: this file, `readme.md`, `just run_*`, the project `CLAUDE.md` and
   `docs/slint_mcp.md`.

Tests: the duplicated tree, `ColumnLimits` and tokenizer tests, plus hex dump, BAF lines and
tree behaviour tests (31), and the UI token lint (2).

Verified through the Slint MCP (headless):

- **BG2**: tree groups, `(O)` overrides and keyboard navigation; BMP; BAM play, pause, cycle and
  frame slider; TIS columns and grid; BCS in Light and Dark; ACM play, pause, stop and end of
  clip; MVE playback; a stub ("2DA Viewer"); an import error; the splitter.
- **BG2EE**: FNT, TTF (font registered at run time, sample edited), PVRZ, WBM.
- **IWD2**: tree and BAM.

Not exercised: the TIS "too large" warning (no tileset in the test games has more than 16,384
tiles), and the clipboard after **Copy** (the headless backend has none; the click is handled).
