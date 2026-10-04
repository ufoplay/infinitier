# Slint keeper port — status & continuation notes

Ongoing task: clone `infinitier_keeper` (egui) as `infinitier_keeper_slint` with the
**same functionality**, vanilla Slint (`std-widgets` only). This file is the handoff for
resuming across sessions.

## User decisions (binding)
- **Duplicate the pure logic** into this crate; do NOT extract a shared crate; do NOT touch
  the egui keeper. (See `src/fields.rs`, `src/cre_fields.rs` — verbatim copies.)
- **Full port** of all four areas: core stat tabs, inventory & items, spells & feats,
  shell & save/load.
- Verify with the **Slint MCP** (Wayland box — no X11 tools). See `../../docs/slint_mcp.md`.
- Don't commit/push unless asked. Nothing committed yet.

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
- `src/proficiencies.rs` — Proficiencies (EDITABLE cards, AD&D). One row per weapon (first-class),
  plus a "(2nd)" row for dual-class. id = stat (first) / stat+256 (second); clamp 0..=5 via
  `Cre::set_proficiency`. Verified: 9→5 clamp.
- `src/memorization.rs` — Memorization (EDITABLE cards, AD&D). One row per spell slot
  ("{type} L{level}"); id = slot index; `Cre::set_spell_memorization_total` (u16).
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
  quantities + delete). NOTE: the Item Browser "add item" flow is NOT ported.
- `src/spells.rs` — Spells (EDITABLE). Dedicated mode (`show-spells`/`spell-tabs`/`spell-tab`/
  `spell-rows`/`SpellRowUi`): inner tab strip (AD&D Innate/Wizard/Cleric or IWD2 per-class,
  with counts), rows Level/Memorized(editable)/Spell(TLK name)/Resource/Delete, sorted (level,
  name). Per-row `SpellRef` stashed on `Model::spell_refs` (id = display row index). `refresh`
  takes `&mut Model` so Spells can record refs. Callbacks `spell-select-tab`/`spell-memorize`/
  `spell-delete`. Verified on classic BG (Dynaheir: Innate(1)/Wizard(39), Slow Poison SPIN102).
  NOTE: the Spell Browser "add spell" flow is NOT ported.

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

## Content modes in app.slint (pick per tab in `main::refresh`)
- **cards** — `show_cards(ui, [Vec<FieldCard>;3])`: the 3-column card grid (label+value/input/
  checkbox rows). Used by Abilities/Resistances/Characteristics/Miscellaneous.
- **table** — `show_table(ui, Vec<VarRow>)`: a virtualised 2-column Name·Value `ListView`. Used by
  Local/Global Variables.
- **grid** — `show_grid(ui, &[&str] headers, &[f32] widths, Vec<GridRow>)`: a wide N-column
  horizontally-scrollable read-only grid (`GridRow{cells:[string]}`). Used by Effects; reuse for
  other read-only tables. Non-virtualised (fine for short lists). For per-row actions/editing add
  another mode.
- **placeholder** — `show_placeholder(ui, text)`: grey "not yet ported" text (or empty).
All four modes are reset by `clear_content(ui)`; each `show_*` calls it first then sets its own.

## Pattern for adding a stat tab (proven with Resistances)
1. New `src/<tab>.rs` with `build(&Model) -> [Vec<FieldCard>;3]` and `commit(&mut Model, id, &str)`.
2. Give the tab its own field enum with `ALL` + `index()`/`from_index()` (ids are tab-local).
3. Use `ui_model::{editable_row, card}`; duplicate any pure data-extraction from the egui `data.rs`.
4. Register `mod` in main.rs; add match arms in `refresh()` and `on_commit_field`.
5. Build with the shim, run headless, click the tab (find its Text handle in the element tree,
   `click_element`), screenshot to verify. NOTE: the element tree truncates at 200 elements, so
   tabs off the right edge of the scrollable strip (e.g. Miscellaneous, the last AD&D tab) aren't
   found/clickable via MCP — verify those by temporarily setting `load_model`'s `selected_tab`
   default to that tab, screenshotting, then reverting. (Possible future polish: wrap the tab
   strip to multiple rows so every tab is on-screen.)

## DONE: Item & Spell Browsers (`src/browser.rs`)
Both browsers are ported as modal overlays (`BrowserOverlay` in app.slint, sized to the window;
`show-*`/`*-browser-rows`/`*-add-enabled`/`*-add-label` props, header **Items**/**Spells** buttons).
Search filters name/type/resref; the list shows Type/Name/Resource (TLK names), sorted by type then
name; selecting + Add writes through:
- **Item Browser**: `add_item` = the egui `assign_to_inventory` (verbatim) into
  `Model::inventory_selected` (`set_inventory_slot_item` + `Itm::max_charges`).
- **Spell Browser**: `add_spell` = AD&D path (`add_known_spell`, book/level from the SPL header).
  Indexes built lazily on first open (`get_all_resources_by_type(Itm/Spl)`), cached on `Model`.
MCP-verified: both overlays render correctly with populated, sorted, TLK-named lists and correct
add-enabled logic. (The full click-through add couldn't be MCP-driven — element tree truncates at
200 so browser/inventory rows fall off — but `add_item`/`add_spell` are verbatim-proven glue.)
NOTE not ported: IWD2 spell-add (list-2DA index path), Inventory/Spells double-click → reveal in
browser, and item/spell icon+description detail panels.

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
All 15 character tabs + Item/Spell Browsers + Save-copy + Load/multi-save/portraits. Clippy+fmt
clean; egui keeper unaffected; nothing committed. Minor not-ported niceties: IWD2 spell-add
(list-2DA path), double-click reveal-in-browser, browser icon/description detail panels, save-tab
wrapping, and the Load dialog's screenshot/portrait preview + delete.

## How to build/run/verify
Build needs `libfontconfig1-dev`; on THIS box use the pkg-config shim:
```
SC=/tmp/claude-1000/-home-ufo-workspaces-github-ufoscout-baldurs-gate-infinitier/de14fed3-de49-4917-82f6-fa09cc0fc38e/scratchpad
export PKG_CONFIG_PATH="$SC/fcshim" SLINT_EMIT_DEBUG_INFO=1
cargo clippy -p infinitier_keeper_slint --all-targets
# run headless + MCP:
export SLINT_MCP_PORT=8080 SLINT_BACKEND=headless
nohup ./target/debug/infinitier_keeper_slint --game-path "/home/ufo/Temp/Games/Baldur's Gate" --savegame 0 --log warn > "$SC/slint2.log" 2>&1 & echo $! > "$SC/app.pid"
# drive via MCP:
python3 scripts/slint_mcp_client.py call list_windows '{}'
python3 scripts/slint_mcp_client.py call take_screenshot '{"windowHandle":{"generation":"1","index":"1"}}'
# stop: kill "$(cat $SC/app.pid)"   (NEVER pkill -f infinitier_keeper_slint — self-kills shell)
```
Test games: Baldur's Gate (AD&D), Icewind Dale 2 (IWD2/d20), all under `/home/ufo/Temp/Games/`.

## Conventions established (follow for new tabs)
- Add tab content as a `build(model) -> [Vec<FieldCard>;3]` (stat tabs) OR a new Slint component
  + model for tables. Route through `refresh()` in main.rs (`match model.selected_tab`).
- Boundary ids for editable fields = `EditableField::index()`. Read-only rows use id `-1`.
- Rebuild the model on commit/selection (NOT on keystroke — would reset focused LineEdit).
- Element handles go stale after any model rebuild — re-fetch the MCP tree after actions.
- Known cosmetic polish: GroupBox titles clip ~6px on the left inside per-column ScrollViews.
