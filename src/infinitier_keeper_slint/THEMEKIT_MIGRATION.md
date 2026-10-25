# Slint keeper → slint-themekit migration plan

**Status: complete (2026-10-25).** No `.slint` file imports `std-widgets`; every design value
outside `ui/themes/` comes from the themekit's tokens (`tests/ui_tokens.rs`). The rest of this
file is the plan as it was carried out, with what each phase did.

Goal: the Slint keeper's UI uses only components from
[slint-themekit](/home/ufo/workspaces/github_ufoscout/slint-themekit) (plus Slint's built-in
elements), styled only by its `Theme` tokens. Done progressively: shell → page entries → tabs →
dialogs. Missing components are added to the themekit along the way.

## Ground rules

- **Themekit changes follow its `CLAUDE.md` and `DESIGN_PRINCIPLES.md`** — every step there:
  one component per file, `Theme` tokens only (literals only in `presets.slint`), controlled
  state (`in` properties + intent callbacks), `StateLayer` for hover/press, `states.disabled`
  fade, `motion` tokens, accessible role/state, export from `ui/lib.slint`, README
  "Components"/"Theming" sections, **gallery demo in the same change** (every variant/state,
  both presets), then `cargo test --workspace`, `cargo clippy --workspace --all-targets`,
  `cargo fmt --all -- --check`. Never import `std-widgets.slint`, never check
  `Theme.current.name`/`.dark` inside a component, never rename/remove a token silently.
- **What goes where.** Generic UI (tabs, lists, select, menu, dialog, spinner) goes into the
  themekit. Keeper-specific composites (field card, floating browser window, resize grip) stay in
  the keeper, built only from themekit components, built-in elements and `Theme` tokens — the
  themekit explicitly supports this (`StateLayer` is public for app-built surfaces).
- **Keeper data flow is unchanged**: the `*Adapter` globals and the Rust `view/`/`callbacks/`
  stay; a migration swaps the widgets inside a page, not its adapter API (except where a
  controlled themekit component simplifies it — e.g. the browser's `in-out all-types`).
- **Each step is shippable**: builds, keeper tests + clippy pass, verified through the Slint
  MCP in both themes, on a BG2 and an IWD2 save. Themekit and keeper changes are separate
  commits.

## What the themekit has today

`Theme` (+ `ThemePresets.light/dark`, `ThemeData`, `Tone`, `TextRole`), `Label`, `Button`
(primary/secondary/ghost/danger, optional icon), `Checkbox` (controlled), `TextField`, `Card`,
`Badge`, `Chip`, `ProgressBar`, `NavItem`, `Divider`, `ScrollArea` (vertical, not virtualised),
`StateLayer`, `Icons` (plus, check, trash, list, chart, contrast). Baseline `cargo test
--workspace` is green.

## Mapping: keeper → themekit

| Keeper today (std-widgets / own) | Themekit | Status |
|---|---|---|
| `Palette.color-scheme` dark/light, keeper `Theme` (muted, error, backdrop, shadow) | `Theme.current = ThemePresets.light/dark`; `Tone.muted`, `Tone.danger`, `palette.shadow` | backdrop needs a **`palette.scrim`** token |
| `Text` with weights/colours | `Label` (`TextRole` + `Tone`) | ✓ |
| `Button` | `Button` (kinds, `Icons`) | ✓ |
| `CheckBox` (incl. two-way `all-types`) | `Checkbox` (controlled) | ✓ — simplifies the browser |
| `LineEdit` (commit on Enter / focus loss) | `TextField` (`accepted`, `out has-focus`) | ✓ |
| `GroupBox` (field cards, Colors) | `Card` + `Label { role: heading }` | ✓ |
| `ScrollView` vertical (cards, appearance, filters, description) | `ScrollArea` | ✓ |
| `ScrollView` both axes (Effects / Journal grid) | `ScrollArea` | **missing: horizontal scrolling** |
| `ChipStrip` (save tabs with ×, character tabs, spell books) | — | **missing: `Tab` + `TabBar`** (+ `Icons.close`) |
| `ListView` + `SelectableRow` (party, browsers, tables, inventory, spells, dialogs) | — | **missing: `ListView` (virtualised) + `ListItem`** |
| `ComboBox` (Attacks, Settings game type) | — | **missing: `Select`** (+ `Icons.chevron-down`) |
| `PopupWindow` (IWD2 spellbook menu) | — | **missing: `Menu` + `MenuItem`** |
| `ModalFrame` (all dialogs) | — | **missing: `Dialog`** (+ `palette.scrim`) |
| `Spinner` (loading overlay) | `ProgressBar` (determinate only) | **missing: `Spinner`** |
| `FloatingWindow`, resize grip | — | keeper composite on tokens |
| `VerticalBox` / `HorizontalBox` | built-in layouts + `Theme.spacing` | ✓ |

Feasibility checked in the Slint 1.18.1 compiler:
- A component **named `ListView`** exposing `content-y`, `content-height`, `content-width`,
  `visible-height`, `visible-width` (lengths) gets the compiler's row virtualisation
  (`object_tree.rs`) — so the themekit can ship a virtualised list without std-widgets.
- `FlexboxLayout` with `flex-wrap: wrap` is a stable built-in → a wrapping `TabBar` (this also
  closes the "wrapping character tab strip" gap).
- A built-in `Tooltip` element (custom-content mode) exists → a themed tooltip is possible
  later (Settings path hover, "Close this save").
- Accessible roles available: `tab`, `tab-list`, `list`, `list-item`, `combobox`, `groupbox`,
  `progress-indicator`; there is no dialog or menu role (use `groupbox` / `button`).

## Themekit additions (just in time, one per change)

API sketches — named for what they mean to the app (principle 9), controlled (principle 5):

| Component | API sketch | Notes |
|---|---|---|
| `Icons.close` ✅, `Icons.chevron-down` | Lucide icons (`x`, `chevron-down`) | added through `icons.slint` + `just icons` (see the themekit's CLAUDE.md "Adding an icon"); never drawn by hand |
| `Tab` | `text`, `selected`, `closable`, `enabled`; `clicked()`, `close()` | role `tab`, item-selected; the × needs an app-given label (e.g. `close-label`) |
| `TabBar` | container: `@children` in a wrapping `FlexboxLayout`, `Theme.spacing` gaps | role `tab-list` |
| `ListView` | `@children` = one `for`; themed scrollbar like `ScrollArea`; the five length properties above | role `list`; spike first |
| `ListItem` | `index` (striping), `selected`, `enabled`; `clicked()`, `double-clicked()`; `out pointer-x/-y`; cells as `@children` | role `list-item`, item-selected; stripe = `surface-variant` |
| `ScrollArea` (change) | add horizontal scrolling (e.g. `horizontal: bool`), both scroll indicators | existing component → update its gallery demo |
| `Select` | `options: [string]`, `current-index` (in), `placeholder`, `enabled`; `selected(int)` | role `combobox`; popup on built-in `PopupWindow` |
| `Menu` + `MenuItem` | `Menu`: `show()` / `close()`, position via `x`/`y`; `MenuItem`: `text`, `enabled`; `clicked()` | items role `button` |
| `Dialog` | `title`; scrim + elevated panel; body = `@children` (button row last) | new token **`palette.scrim`** (zero = transparent: still usable); role `groupbox` |
| `Spinner` | indeterminate, `tone` | `motion.normal` loop; static when durations are `0ms` (principle 3) |

Possible (decide when reached): a **`Label` emphasis** option for bold values in field cards
(today weight comes from `TextRole`); a **dense list-row size token** (lists use
`control-height` = 36px vs the keeper's 24–30px rows); `Tooltip`; `Avatar` (initials, for an
egui-style party rail).

## Phases

### Phase 0 — Foundation (no visible change) — ✅ 2026-10-25 (except step 1)
Done: path dependency, `build.rs`, `tests/ui_tokens.rs` (all 25 non-trivial files listed, grouped
by phase), `Theme.current` follows the Light/Dark toggle, spike passed. Also added
`.vscode/settings.json` (`slint.libraryPaths` → the themekit checkout) so the editor resolves
`@themekit`. Spike results (software renderer, headless MCP): a `ListView` built from
`Flickable` with the five length properties instantiated **16 of 3,000** rows and scrolled to
rows 2984–2999; a `FlexboxLayout` wrapped the 13 character tabs onto 5 rows; a `PopupWindow`
showed themed content.

1. Themekit: `CLAUDE.md` says `.cargo/config.toml` sets `SLINT_COMPILER_DENY_WARNINGS`, but the
   file doesn't exist — add it (or fix the doc). Likewise the README mentions a
   `.vscode/settings.json` the repo doesn't have. *Waiting for confirmation.*
2. Keeper `Cargo.toml`: `slint-themekit` as **build-dependency** (+ **dev-dependency** for the
   lint). Path dependency to the sibling checkout while developing; switch to a git dependency
   pinned to a pushed rev before pushing (CI/release can't see the sibling path) — same
   convention as `egui-components` in the workspace `Cargo.toml`.
3. Keeper `build.rs`: `compile_with_config` with `slint_themekit::library()` as library path.
4. Keeper `tests/ui_tokens.rs`: run `slint_themekit::lint::scan_dir` on `ui/`, with a
   `NOT_YET_MIGRATED` exemption list holding every current file. Each step removes the files it
   migrates; the list must be empty at the end. Same test asserts no migrated file imports
   `std-widgets.slint`.
5. Theme: `app.slint` imports `Theme`/`ThemePresets` from `@themekit`; the Light/Dark toggle
   assigns `Theme.current` (apps may read `Theme.current.dark`), and keeps
   `Palette.color-scheme` in sync while std-widgets remain.
6. **Spike** (throwaway branch): a themekit-style `ListView` with 3,000 rows, a `FlexboxLayout`
   wrap and a `PopupWindow`, in the keeper's software renderer + headless MCP. Confirms the
   design before any component is written.

### Phase 1 — Application shell — ✅ 2026-10-25
`app.slint` (background, layouts on `Theme.spacing`), `shell/header_bar.slint` (`Button`s;
theme toggle as ghost icon button with `Icons.contrast`; name/status as `Label`s),
`shell/empty_state.slint` (`Label`s), `shell/save_tab_strip.slint`.
Themekit: **`Tab`, `TabBar`, `Icons.close`**.

Done:
- Themekit: `Tab` (`text`, `selected`, `closable`, `close-label`; `clicked()`,
  `close-requested()`; role `tab`, the × a `button` named by `close-label`), `TabBar` (wrapping
  `FlexboxLayout`, role `tab-list`), `Icons.close`; README and gallery ("Tabs" card: selection,
  closable tabs wrapping in a narrow bar, reopen). `cargo test --workspace` and
  `just check_code` green; checked headless in Light and Dark.
- Keeper: header buttons follow the egui keeper (Load / Save / Items / Spells primary, theme
  toggle and Settings ghost); save tabs are closable `Tab`s; the window's sizes live in
  `ui/themes/metrics.slint` (`Metrics`, exempt from the lint); the std-widgets Palette sync moved
  to `legacy_palette.slint` (Phase 5) so `app.slint` no longer imports std-widgets.
  `components/chip_strip.slint` stays until its last user, the Spells tab (Phase 3).
- Learned: an invisible (`visible: false`) item still takes its space in a layout, so a closed
  tab must leave the model (the keeper does; the gallery uses one `if` per tab, since Slint can
  neither filter a model nor nest `if` directly in `for`).

### Phase 2 — Page entries — ✅ 2026-10-25
The main window's regions and the windows opened from the header:
1. `shell/character_panel.slint` tab strip → `TabBar`/`Tab` (now wrapping: every tab visible).
2. `shell/party_list.slint` → **`ListView` + `ListItem`** (themekit).
3. Browsers: `components/floating_window.slint` rebuilt on tokens (title bar, icon-only close
   `Button`, grip); `browser/browser_window.slint` → `TextField`, controlled `Checkbox` (drop the
   `in-out all-types` two-way binding), `ListView`/`ListItem`, `Label`, `ScrollArea`; spellbook
   popup → **`Menu` + `MenuItem`** (themekit).
Keeper: `components/chip_strip.slint` and `components/list.slint` (`SelectableRow` etc.) retire
once their last user is gone.

Done:
- Themekit: `ListView` (virtualised; its scroll indicator shared with `ScrollArea` through a
  private `ScrollIndicator`), `ListItem` (controlled; content-sized rows, stripes by `index`,
  `clicked()` / `double-clicked()`, `pointer-x/-y`), `Menu` (a `PopupWindow`) + `MenuItem`;
  README and gallery ("Lists": 1,000 rows, selection, double-click, menu with a disabled item).
  `cargo test --workspace` and `just check_code` green; checked headless in Light and Dark.
- Keeper: the character tab strip is a `TabBar` (all tabs visible, wrapping onto a second row —
  the egui keeper's behaviour); the party list is a `ListView`; the browsers are on themekit
  components (controlled `Checkbox`es, so `BrowsersAdapter.*-all-types` became plain `in`
  properties; `TextField`; `Menu` for the IWD2 spellbooks). New `Metrics`: party rail and
  portraits, browser placement / sizes / columns / row height. `components/list.slint` moves to
  Phase 4 (the Load / Settings dialogs still use it).
- Verified on BG2 + IWD2: party selection, tab wrap + switching, search, type filters (master and
  per type), selection + detail, arrow keys, reveal from Inventory / Spells, add by double-click,
  close, IWD2 menu from Add and from a double-click.
- Learned: `Math.mod(-1, 2)` is 1 in Slint (the sign follows the divisor), so `ListItem` only
  stripes when `index >= 0`; a browser row has a fixed height (`Metrics.browser-row-height`) so
  scrolling to a row doesn't depend on the list having been laid out yet.

### Phase 3 — Tabs — ✅ 2026-10-25
1. **Card tabs** (`components/field_card.slint` + `tabs/cards_tab.slint`): `Card`, `Label`,
   `TextField` (commit on `accepted` / `has-focus` loss), `Checkbox`, **`Select`** (Attacks).
   Verify each of the 8 tabs: Abilities, Levels & Kits, Characteristics, Feats, Memorization,
   Proficiencies, Resistances, Miscellaneous.
2. **Appearance**: `Card`, `Label`, `ScrollArea`.
3. **Local / Global Variables** (`tabs/table_tab.slint`): `ListView`, `ListItem`, `Label`.
4. **Effects / Journal Entries** (`tabs/grid_tab.slint`): **`ScrollArea` horizontal** (themekit
   change), `Label` (wrapping journal text).
5. **Inventory**: `ListView`/`ListItem` (slot select + double-click reveal), `TextField`,
   `Button` (`Icons.trash`).
6. **Spells**: `TabBar`/`Tab` (books), `ListView`/`ListItem`, `TextField`, `Button`.

### Phase 4 — Dialogs — ✅ 2026-10-25
Themekit: **`Dialog`** (+ `palette.scrim`), **`Spinner`**. Then, one per step: Loading overlay
(`Dialog` + `Spinner`, keeps blocking keys), Confirm, Save (`TextField`), Load (`ListView`/
`ListItem`, screenshot + portraits), Settings (`ListView`/`ListItem`, `TextField`, `Select`).
Keeper: `components/modal_frame.slint` and the keeper `theme.slint` retire.

### Phase 5 — Done — ✅ 2026-10-25
`NOT_YET_MIGRATED` empty, no `std-widgets.slint` import anywhere, `Palette` sync removed. Update
`PORT_STATUS.md`, the readme and `docs/slint_mcp.md` (accessible labels/roles change). Optional:
`Tooltip`, egui-style party rail (`Avatar`), a keeper-specific theme (`ThemeData`).

### Phases 3–5: what was done
- Themekit: `Select` (controlled dropdown on `Menu`, `Icons.expand` = Lucide `chevron-down`),
  `ScrollArea` scrolls sideways when its content is wider (a horizontal indicator; the private
  `ScrollIndicator` gained an orientation), `Dialog` (fits its content or a given size, over the
  new `palette.scrim` token), `Spinner` (an arc driven by `animation-tick()`, timed from
  `motion.normal`, still when animation is off). README, gallery (selects incl. disabled +
  placeholder, a grid scrolling both ways, spinners, a dialog); tests and `just check_code` green.
- Keeper tabs: field cards on `Card` / `TextField` / controlled `Checkbox` / `Select` (the
  checkbox commits the opposite of the row's value); Appearance on `ScrollArea` + `Card`;
  Variables, Inventory, Spells on `ListView` / `ListItem` (inventory slot selection and
  double-click reveal kept; Delete is an icon button announced "Delete"); Effects / Journal on
  the two-way `ScrollArea`; spellbooks on `TabBar` / `Tab`.
- Keeper dialogs: Settings, Load (+ delete prompt), Save, Confirm and Loading on `Dialog`;
  `Select` for the game type; `Spinner` while loading (keys still blocked).
- Removed: `components/{list,chip_strip,modal_frame}.slint`, the keeper `theme.slint` and the
  transitional `legacy_palette.slint`. `tests/ui_tokens.rs` lost its migration list: it now
  lints all of `ui/` except `themes/` and rejects any `std-widgets.slint` import.
- Verified on BG2 + IWD2 in Light and Dark: every tab, a clamped field commit, the Attacks select,
  inventory quantity / reveal / delete, spell memorize / delete, IWD2 kit checkboxes both ways,
  all dialogs (incl. the delete prompt and the loading spinner).

## Icons
The themekit's icons are Lucide's (https://lucide.dev, ISC), vendored unmodified at the release
pinned in its `just/icons.just`; `just icons` downloads every icon `ui/icons.slint` references and
a test rejects anything else. A missing icon is a new `Icons` property pointing at a Lucide file,
not an SVG written by hand.

## Keeper-specific design values
Values that are the keeper's own layout, not theme roles (the 220px party rail, the browsers'
column widths, the window's preferred size), go into a keeper global under `ui/themes/` — the
place the themekit README suggests for app theme files — exempted from the lint permanently.
Create it when the first migrated file needs it.

## Per-step checklist
1. List the std-widgets / keeper widgets the piece uses; map each to the table above.
2. Missing component → themekit change first, per its `CLAUDE.md` (gallery included), all its
   commands green.
3. Migrate the keeper piece; remove it from `NOT_YET_MIGRATED`.
4. `cargo fmt`, `cargo clippy -p infinitier_keeper_slint --all-targets --no-deps -D warnings`,
   `cargo test -p infinitier_keeper_slint`.
5. MCP: the piece's interactions in Light and Dark, BG2 + IWD2; screenshots compared with before.
6. Note the step in this file (✅ + date).

## Risks and decisions to confirm
- **Virtualised `ListView`** relies on the compiler's name-based detection — confirmed in the
  1.18.1 source, to be proven by the Phase 0 spike before building on it.
- **Density**: themekit controls are 36px high; the keeper's lists are 24–30px. Either accept
  taller rows or add a size token (it must be zero-safe, principle 3).
- **Mixed look** between phases (std Fluent + themekit) is expected; colours stay in step via
  the synced dark flag.
- **Accessible labels/roles change** (e.g. tabs become `tab`, lists `list-item`): MCP test
  scripts follow along.
- **Dependency**: the path dependency must become a pinned git dependency before the keeper is
  pushed, or CI/release builds fail.
