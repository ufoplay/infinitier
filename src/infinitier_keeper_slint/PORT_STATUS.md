# Slint keeper port — status & continuation notes

Ongoing task: clone `infinitier_keeper` (egui) as `infinitier_keeper_slint` with the
**same functionality** in Slint — now built on slint-themekit (see `THEMEKIT_MIGRATION.md`),
originally vanilla `std-widgets`. This file is the handoff for
resuming across sessions.

## User decisions (binding)
- **Duplicate the pure logic** into this crate; do NOT extract a shared crate; do NOT touch
  the egui keeper. (See `src/components/editable_fields.rs`, `src/components/cre_fields.rs` —
  verbatim copies.)
- **Full port** of all four areas: core stat tabs, inventory & items, spells & feats,
  shell & save/load.
- Verify with the **Slint MCP** (Wayland box — no X11 tools). See `../../docs/slint_mcp.md`.
- Don't commit/push unless asked. Nothing committed yet.

## Code layout (modularised 2026-10-25)
Split like Slint's demo apps (gallery / energy-monitor / printerdemo) and like the egui keeper's
`components/`, `ui/` and `ui/tabs/`. Each UI part owns its structs and an exported `*Adapter`
global that the Rust side fills (`ui.global::<XAdapter>().set_*`) and handles (`on_*`); reusable
widgets take properties in and send callbacks out. `ui/app.slint` only composes the parts and
re-exports the adapters / structs for Rust.

| `ui/` (Slint) | `src/` (Rust) | egui keeper |
|---|---|---|
| `app.slint` (window, Light / Dark), `themes/metrics.slint` (the keeper's layout values) | `main.rs`, `app.rs`, `state.rs` | `main.rs`, `app.rs`, `state.rs` |
| `components/` — `FloatingWindow`, `FieldCardView` (on themekit components) | `components/` — `editable_fields.rs`, `cre_fields.rs`, `field_card.rs` (row builders) | `components/` |
| `shell/` — `ShellAdapter`, `HeaderBar`, `SaveTabStrip`, `PartyList`, `EmptyState`, `CharacterPanel` (+ `CharacterContent`) | `view/shell.rs`, `view/character.rs`, `callbacks/shell.rs`, `callbacks/character.rs` | `ui/header_panel.rs`, `ui/save_tab_strip.rs`, `ui/character_panel.rs`, `components/party_selector.rs` |
| `tabs/` — one page per layout: `CardsTab` (8 stat tabs), `TableTab` (variables), `GridTab` (Effects, Journal), `AppearanceTab`, `InventoryTab`, `SpellsTab` | `tabs/` — one module per tab + the `CharacterTab` enum | `ui/tabs/` |
| `browser/` — `BrowserWindow` (a `FloatingWindow`), `BrowserLayer` + `BrowsersAdapter` | `browser/`, `view/browsers.rs`, `callbacks/browsers.rs` | `ui/item_browser.rs`, `ui/spell_browser.rs` |
| `dialogs/` — Settings, Load (+ `DeleteSaveDialog`), Save, `ConfirmDialog`, `LoadingOverlay` | `dialogs/`, `view/dialogs.rs`, `callbacks/dialogs.rs` | `ui/*_action.rs` |

Each folder has a barrel file (`components.slint`, `shell.slint`, `tabs.slint`, `browser.slint`,
`dialogs.slint`) re-exporting its parts. Gotchas found while splitting:
- A global's callback can't be implemented in .slint ("Can't assign a local callback handler to
  an alias to a global callback"), so the browser list's scroll is the one window-level API:
  `AppWindow.browser-scroll-to(kind, row)` (Rust: `invoke_browser_scroll_to`).
- A `PopupWindow` among a component's children is positioned relative to the component root,
  not to the `@children` container it lands in (the browser's spellbook menu).
- Widgets come from slint-themekit (`@themekit`), design values from its `Theme` tokens; the
  keeper's own sizes live in `themes/metrics.slint` (`tests/ui_tokens.rs` enforces both).

The DONE log below predates the restructure: its file paths and property names are historical.

## DONE (working + MCP-verified)
- Crate scaffold, workspace member, build.rs, software-renderer+winit, `mcp` feature.
- `src/fields.rs` — full `EditableField` (60 fields) parse/clamp/write logic, `AttacksOption`,
  class-level summing. Boundary id = **index into `EditableField::ALL`** (`.index()`/`.from_index()`).
- `src/cre_fields.rs` — GAM party gold/reputation.
- `src/state.rs` — `CharacterTab` enum (+ `is_visible_for_game`, `label`), `Model`
  (single save tab for now), `visible_tabs()`, selected_cre[/_mut].
- `src/ui_model.rs` — shared `editable_row()` / `card()` `FieldRow`/`FieldCard` constructors.
- `src/abilities.rs` — Abilities tab: `build()` (3 columns, all sections, live effective
  THAC0/AC/MaxHP/Lore/thief previews, Attacks dropdown) + `commit()`/`commit_attacks()`.
- `src/resistances.rs` — Resistances tab: `build()` + `commit()`. Duplicated `resist_data`,
  `RField` enum (id↔setter, index into `RField::ALL`), per-int-type clamp. AD&D (resistances +
  AC modifiers + 5 saves) and IWD2 (12 resistances incl. Magic Damage + 3 d20 saves).
- `src/characteristics.rs` — Characteristics tab (READ-ONLY, no `commit`). Duplicated the whole
  `data.rs` (CharData/resolve + IDS/2DA/TLK resolution helpers). Identity card (IDS-resolved
  gender/race/alignment/class/kit/racial/state + Fallen/Dual-Class checkboxes), Kill Stats card
  (GAM `char_stats`, TLK-resolved strongest-kill name), Miscellaneous card (`CreatureFlags::MISC`
  disabled checkboxes). TLK strrefs resolved at build time (no per-frame memo needed).
- `src/miscellaneous.rs` — Miscellaneous tab (READ-ONLY). Duplicated `MiscData`/`misc_data`:
  "Other" card (V1.0 CRE header: turn-undead, tracking, identifier, death var, 5 scripts) + three
  Day/Hour/Minute time cards (GAM `game_time` / `join_time`).
- `src/variables.rs` — Local & Global Variables tabs (READ-ONLY). Name·Value tables rendered via a
  **virtualised Slint `ListView`** (new content mode — see `show-table`/`table-rows`/`VarRow` in
  app.slint). Local = `cre.local_variables()`; Global = GAM `variables` sorted with EEKeeper's
  collation (`collation_key`, `_`→`@`), duplicated from the egui `data.rs`. Global list ~2000 rows.
- `src/effects.rs` (+ `src/effects/opcode_names.rs`, copied verbatim) — Effects tab (READ-ONLY).
  Wide 11-column grid via the new **grid content mode** (`show-grid`/`grid-headers`/`grid-widths`/
  `grid-rows`/`GridRow` in app.slint; horizontally scrollable, non-virtualised — effect lists are
  short). Duplicated `effect_rows` (EE V2 effects only, excludes op233/op187), opcode→name table,
  SPELL.IDS resref resolution, timing/target label maps. NOTE: only EE (V2) effects show; classic
  BG (V1) yields an empty table — verified structurally on classic BG (headers only). To see
  populated rows, load an EE/V2 save whose saves are discoverable in the game folder.
- `FieldRow` extended with `is_check`/`checked` → `ui_model::check_row` renders a disabled
  CheckBox row; `ui_model::readonly_row` added and now shared by abilities + resistances +
  characteristics + miscellaneous (abilities' local `read_only_row`/`card` removed).
- `src/journal.rs` (+ `src/journal/calendar.rs`) — Journal Entries (READ-ONLY). 4-col wrapping
  grid (`grid-wrap`); TLK text + Harptos calendar duplicated. (Time column shows the calendar
  template on classic BG — an EE-tuned data quirk shared with the egui keeper.)
- `src/appearance.rs` (+ `src/appearance/palette.rs`) — Appearance (READ-ONLY). New **appearance
  content mode** (`show-appearance`/`animation-name`/`color-swatches`/`ColorSwatch{label,image}`):
  ANIMATE.IDS name + 7 palette-BMP gradient swatches rendered as Slint `Image`s via
  `SharedPixelBuffer`.
- `src/proficiencies.rs` — Proficiencies (EDITABLE table, AD&D). EEKeeper's Proficiency · First
  Class · Second Class table; the second class is editable only for a dual-classed character's
  weapons (blank when zero elsewhere). id = stat (first) / stat+256 (second); clamp 0..=5 via
  `Cre::set_proficiency`. Verified: 9→5 clamp.
- `src/memorization.rs` — Memorization (EDITABLE table, AD&D). EEKeeper's Type · Level · Max Can
  Memorize table, one row per spell slot; id = slot index; `Cre::set_spell_memorization_total`
  (u16).
- `src/levels.rs` — Levels & Kits (EDITABLE cards, IWD2). Left: 11 class-level `EditableField`
  rows + Total; right: editable kit checkboxes (`Iwd2Kits`). id = EditableField index / kit toggle
  `1000+i`; class levels route to `abilities::commit`, kits flip `set_iwd2_kits`.
- `src/feats.rs` — Feats (EDITABLE cards, IWD2). One editable Known checkbox per feat + a
  conditional Level input for known stackable feats. Names via FEATS.IDS→feats.2da→tlk. id =
  feat_index (known) / `1000+slot` (level).
- `ui_model::editable_check_row` + app.slint CheckBox now toggles `commit-field(id,"1"/"0")` when
  `id>=0` (editable); `id==-1` stays disabled (read-only, Characteristics).
- **IWD2 verified** (Icewind Dale 2 save): tab visibility (Levels/Feats shown, Memo/Prof hidden),
  Abilities d20 skills + "to hit" + morale-disabled + per-class levels, Characteristics
  (subrace/aligns-2da/class-from-levels), Resistances d20 saves, Levels & Kits, Feats.
- `src/inventory.rs` — Inventory (EDITABLE). Dedicated content mode (`show-inventory`/`inv-rows`/
  `InvRow`): BAM-decoded icons (Slint `Image`), position, editable quantity triple, TLK item name,
  resref, Delete, and row selection (`Model::inventory_selected`, the Item Browser target).
  Callbacks `inv-select`/`inv-quantity`/`inv-delete`. Verified on classic BG (icons + names +
  quantities + delete). Double-clicking a filled slot reveals its item in the Item Browser.
- `src/spells.rs` — Spells (EDITABLE). Dedicated mode (`show-spells`/`spell-tabs`/`spell-tab`/
  `spell-rows`/`SpellRowUi`): inner tab strip (AD&D Innate/Wizard/Cleric or IWD2 per-class,
  with counts), rows Level/Memorized(editable)/Spell(TLK name)/Resource/Delete, sorted (level,
  name). Per-row `SpellRef` stashed on `Model::spell_refs` (id = display row index). `refresh`
  takes `&mut Model` so Spells can record refs. Callbacks `spell-select-tab`/`spell-memorize`/
  `spell-delete`. Verified on classic BG (Dynaheir: Innate(1)/Wizard(39), Slow Poison SPIN102).
  Double-clicking a row reveals its spell in the Spell Browser.

## ALL 15 CHARACTER TABS DONE ✅
Abilities, Levels & Kits, Characteristics, Appearance, Inventory, Spells, Feats, Memorization,
Proficiencies, Resistances, Effects, Local/Global Variables, Journal Entries, Miscellaneous —
all ported, clippy+fmt clean, verified on classic BG (AD&D) and Icewind Dale 2 (IWD2/d20). The
`refresh` match is now exhaustive (no placeholder fallback).
- `ui/app.slint` — header (Save+status), party ListView, horizontally-scrollable tab strip,
  generic `CharacterCard`/`FieldRow`/`FieldCard` model, 3-column card grid inside ONE outer
  vertical ScrollView (fixes the GroupBox-title left-clip), placeholder for unported tabs.
  Label flexes+elides so inputs never clip. Commit-on-blur + Enter, Attacks ComboBox.
- `src/main.rs` — load pipeline; `refresh()` dispatches cards per active tab; `on_commit_field`
  dispatches to the active tab's `commit()` (**field ids are tab-local**). Wiring: select-party,
  select-tab, commit-field, commit-attacks, save copy+export.
- Verified via MCP against `/home/ufo/Temp/Games/Baldur's Gate` savegame 0 (Palagorn party):
  Abilities (values + previews + layout), tab visibility (AD&D shows Memorization/Proficiencies,
  hides Levels/Feats), tab switching, Resistances (correct save values, both cards), title-clip fix.

## Character pages (`CharacterAdapter.content`)
`view/character.rs` picks the page per tab and fills its adapter:
- **cards** — `CardsAdapter.column0..2` (`[FieldCard]`) + the Attacks dropdown. Abilities, Levels &
  Kits, Characteristics, Feats, Resistances, Miscellaneous.
- **edit-table** — `EditTableAdapter` (`name-header`, `headers`, `[EditRow]` of `[EditCell]`), a
  striped table whose value cells may be editable; `commit(id, value)` goes to the active tab.
  Memorization, Proficiencies.
- **table** — `TableAdapter.rows` (`[VarRow]`), a virtualised Name · Value list. Local / Global
  Variables.
- **grid** — `GridAdapter` headers / widths (and their sum) / rows (`GridRow{cells}`) / wrap: a wide
  read-only table, virtualised, that scrolls sideways under a header following the list's
  `content-x`. Effects, Journal Entries (wrapped).

Every table (these pages, the Load and Settings dialogs, the browsers) is a `TableHeader` of
`ColumnHeader`s (`ui/components/table.slint`) over a `ListView` of striped `ListItem`s, so they all
look alike.
- **appearance**, **inventory**, **spells** — their own pages and adapters.
- **none** — no character selected.

## Pattern for adding a stat tab (proven with Resistances)
1. New `src/tabs/<tab>.rs` with `build(&Model) -> [Vec<FieldCard>;3]` and `commit(&mut Model, id, &str)`.
2. Give the tab its own field enum with `ALL` + `index()`/`from_index()` (ids are tab-local).
3. Use `components::field_card::{editable_row, card}`; duplicate any pure data-extraction from the
   egui `data.rs`.
4. Add it to `tabs/mod.rs` and `CharacterTab`; add match arms in `view/character.rs` and in
   `callbacks/character.rs`'s commit dispatch.
5. Build, run headless, click the tab, screenshot to verify (see `../../docs/slint_mcp.md` for
   the driving gotchas). Tabs scrolled off the right edge of the strip need the strip scrolled
   first: drag its `ScrollBar::thumb` (dragging a chip doesn't scroll).

## DONE: Item & Spell Browsers (`src/browser/`, `ui/browser.slint`) — egui parity (2026-10-25)
- **Floating windows** (`BrowserWindow`): non-modal panels over the editor, dragged by the title
  bar, resized from the bottom-right grip, closed by ×; any interaction raises a window over the
  other one. Header **Items**/**Spells** toggle them and need only a loaded game.
- **State** (`browser::Browsers` on `App`, reset on a game switch): one `Browser` per kind — the
  lazily built index, free text, hidden types (excluded set, like egui), selection + its resolved
  icon/description, and a pending scroll. Survives opening/closing saves.
- **Index** (`browser/items.rs`, `browser/spells.rs`, duplicated from the egui browsers): items
  Type/Name/Resource sorted by type+name; spells Type/Lvl/Name/Resource/Script Name (egui type
  names incl. "Cleric", `SPELL.IDS` script names) sorted by type+level+name. Text matches type,
  name, resref and script name.
- **Filters**: count ("Items: N"), search, "Types" master checkbox + one per type.
- **Detail**: selected entry's BAM icon (`icons::bam_icon`) + TLK description ("(no
  description)" / "Select an item to see its description.").
- **Add**: button or row double-click. Items go into the Inventory tab's selected slot (needs the
  Inventory tab, an embedded CRE and a slot), labelled "Add to {name} inventory". AD&D spells use
  the SPL header's book/level; IWD2 spells open a `PopupWindow` menu of
  `ImportedSpl::iwd2_placements` ("{book} — Lvl {level}", cached per selection), at the button or
  at the double-click.
- **Reveal**: double-click a filled inventory slot / a Spells-tab row → open the browser, select
  the (lower-cased) resref, clear the text, re-show its type, scroll to it.
- **Keys**: Up/Down move the selection while the list has focus (the list follows).
- Callbacks carry the kind (`browser-*(0|1, …)`); `view::refresh_browsers` pushes
  `BrowserState` + rows/types (replaced only when changed) and calls `*-browser-scroll-to`.
- `inv-select` only repaints the browsers (the Slint side highlights `inv-selected` itself), so
  the slot rows survive the first click and the double-click registers.
- MCP-verified on BG2 (search, types, arrows, detail, add item/spell by double-click and button,
  reveal from Inventory/Spells, drag/resize/raise, dark theme) and IWD2 (spellbook menu from the
  button and from a double-click, Paladin add, reveal of an upper-case resref).

## DONE: multi-save shell + portraits
- **Multi-save** (`state.rs`): the "park" design — `Model` keeps the active save's fields flat
  (so no tab module changed) plus `parked: Vec<Option<SaveState>>` (`parked[active_tab]` is always
  None) + `active_tab`. `switch_save` swaps active↔parked via `mem::replace`; `open_save` appends;
  `close_save` removes (keeps ≥1). MCP-verified: loading a 2nd save shows the tab strip, and
  switching swaps the whole party + character data (Auto-Save L7/party-of-6 ↔ 001-Start L1/party-of-1).
- **Save-tab strip** (app.slint, below the header, shown when >1 save): a chip per save (name + ×
  close), highlight active; `select-save-tab`/`close-save-tab`.
- **Load dialog** (`browser::load_rows` + a 3rd `BrowserOverlay`): header **Load** button opens a
  modal listing discovered saves; select + Load = `load_save_into_tab` (import GAM + `ImportedGam::load`
  + `open_save`). MCP-verified: modal lists all saves; selection enables Load.
- **Portraits** (`main::party_portrait`/`load_bmp_image`): party-list entries are now
  `PartyMember{name, portrait}` — the CRE's small/large portrait BMP decoded to a Slint image.
  MCP-verified rendering.

## PORT COMPLETE ✅
All 15 character tabs + Item/Spell Browsers + Save + Load/multi-save/portraits, plus the shell
below. Clippy+fmt clean; egui keeper unaffected.

## DONE: shell parity with the egui keeper (2026-10-24)
The egui keeper gained a settings-driven shell; ported here (logic duplicated, not shared):
- **No CLI game**: args are `--config` (default `infinitier_keeper.toml`, same TOML format as the
  egui keeper — `src/settings.rs` is a verbatim copy) and `--log`. Startup loads the settings'
  `active` game.
- **Top-level state** (`src/app.rs`): `Session::{NoGame, Game(LoadedGame), Saves(Model)}`.
  `Model` is unchanged for the tab modules (a game + ≥1 open save); opening the first save turns
  `Game` into `Saves` (`Model::new`), closing the last turns it back (`Model::into_game`).
- **Game loading/switching** on a `game-load` thread, polled by a `slint::Timer`; a modal
  `LoadingOverlay` with a `Spinner` blocks the UI (backdrop + key-swallowing FocusScope). A failed
  load (or a loader panic) reopens Settings with the error.
- **Settings dialog** (`src/settings_dialog.rs` + `ui/dialogs.slint`): games list with
  autodetected short-name types (background detection), add form (validates CHITIN.KEY), Remove,
  select + Close / double-click to switch; `active` + `theme` persisted.
- **Dark/Light** toggle: `dark-theme` → `Palette.color-scheme`; hard-coded colours in app.slint
  replaced by Palette brushes.
- **Empty states** "No game loaded" / "No save loaded"; header buttons gated (Load needs a game;
  Save needs a save, Items/Spells a game); the last save tab can be closed.
- **Save dialog** (`src/save_dialog.rs`): editable name pre-filled with the next free
  `(Edited NNNN)`; the active tab follows the new folder.
- **Load dialog** (`src/load_dialog.rs`): newest first with "Time Saved", screenshot + portrait
  preview, double-click to open, Delete with a confirmation dialog.
- `main.rs` split: `view` (UI push), `callbacks` (wiring) — since split per UI part, see Code layout.
- **Gotcha**: list properties are only replaced when their rows change (`view::rows_changed`) —
  a new model rebuilds the rows and breaks double-click detection.

## Still missing vs the egui keeper
Nothing functional. The character tab strip now wraps (themekit `TabBar`, see
`THEMEKIT_MIGRATION.md`). Presentation differences remain: the egui party rail (large portrait,
counter, slider), its empty-slot / external-CRE messages, and tooltips.

## How to build/run/verify
Build needs `libfontconfig1-dev`; on THIS box use the pkg-config shim:
```
SC=/tmp/claude-1000/-home-ufo-workspaces-github-ufoscout-baldurs-gate-infinitier/de14fed3-de49-4917-82f6-fa09cc0fc38e/scratchpad
export PKG_CONFIG_PATH="$SC/fcshim" SLINT_EMIT_DEBUG_INFO=1
cargo clippy -p infinitier_keeper_slint --all-targets
# run headless + MCP:
export SLINT_MCP_PORT=8080 SLINT_BACKEND=headless
nohup ./target/debug/infinitier_keeper_slint --config config_test/infinitier_keeper.toml --log warn > "$SC/slint2.log" 2>&1 & echo $! > "$SC/app.pid"
# drive via MCP:
python3 scripts/slint_mcp_client.py call list_windows '{}'
python3 scripts/slint_mcp_client.py call take_screenshot '{"windowHandle":{"generation":"1","index":"1"}}'
# stop: kill "$(cat $SC/app.pid)"   (NEVER pkill -f infinitier_keeper_slint — self-kills shell)
```
Test games: Baldur's Gate (AD&D), Icewind Dale 2 (IWD2/d20), all under `/home/ufo/Temp/Games/`.

## Conventions established (follow for new tabs)
- Stat tabs: `build(model) -> [Vec<FieldCard>;3]` shown by `CardsTab`. Anything else: a new page in
  `ui/tabs/` with its own struct + `*Adapter`, a `CharacterContent` variant, and a `show_*` in
  `view/character.rs`.
- A new UI part: its `.slint` file owns its structs + adapter; export them through the folder's
  barrel and re-export from `app.slint`; fill it in a `view/` module, handle it in a `callbacks/`
  module. Keep `components/` free of adapters.
- Boundary ids for editable fields = `EditableField::index()`. Read-only rows use id `-1`.
- Rebuild the model on commit/selection (NOT on keystroke — would reset focused LineEdit).
- Element handles go stale after any model rebuild — re-fetch the MCP tree after actions.
