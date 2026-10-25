# Driving the Slint apps with the Slint MCP

Both Slint GUIs, `infinitier_keeper_slint` and `infinitier_explorer_slint`, have the same `mcp`
feature; the examples use the keeper, the explorer runs the same way with its game folders as
arguments.

Slint ships an **embedded MCP server** behind the `slint/mcp` cargo feature. When enabled and
switched on at runtime it opens an HTTP endpoint that an external tool (or an AI agent) can use to
read the app's **element tree**, **inject input**, and **capture screenshots** — the Slint analogue
of the [egui inspection port](egui_mcp.md).

## One-time setup (already done in this repo)

1. **The `mcp` feature** — `src/infinitier_keeper_slint/Cargo.toml` gates it:
   ```toml
   [features]
   mcp = ["slint/mcp"]
   ```
   It is a dev-only opt-in; a normal `cargo run -p infinitier_keeper_slint` has no MCP surface.

2. **Debug info at build time** — the element tree needs source locations, so build with
   `SLINT_EMIT_DEBUG_INFO=1` (see below).

3. **Register it with Claude Code** (already in `~/.claude.json` for this project):
   ```sh
   claude mcp add --transport http slint-keeper http://127.0.0.1:8080/mcp
   ```
   > Registering mid-session does **not** expose the tools to that session — they appear on the
   > next run (or after `/mcp` → reconnect). The endpoint is only live while the app runs with
   > `SLINT_MCP_PORT` set, so the server shows as *not connected* the rest of the time.

## Running the app with the MCP on

```sh
SLINT_EMIT_DEBUG_INFO=1 \
SLINT_MCP_PORT=8080 \
SLINT_BACKEND=headless \
cargo run -p infinitier_keeper_slint --features mcp -- \
    --config config_test/infinitier_keeper.toml
# serves Streamable HTTP at http://127.0.0.1:8080/mcp
```

- `SLINT_BACKEND=headless` renders frames **without a compositor** — essential on this Wayland box,
  where a native window is invisible to X11 tooling. Screenshots still work (they come back through
  the MCP, not the screen).
- Drop `SLINT_BACKEND=headless` to drive a real on-screen window instead.

Check it's up: `ss -ltn | grep 8080`.

## Using it

The transport is **Streamable HTTP** (JSON-RPC 2.0 over HTTP POST, responses as JSON or an SSE
`data:` stream). Any MCP client that speaks it works; a minimal stdlib-only Python client lives at
`scripts/slint_mcp_client.py` for ad-hoc probing.

| tool | what it does |
|---|---|
| `list_windows` | enumerate open windows → `windowHandle` |
| `get_window_properties` | read one window by handle |
| `get_element_tree` | full element tree under an `elementHandle` |
| `get_element_properties` | read one element's properties |
| `find_elements_by_id` | locate elements by their `.slint` id |
| `query_element_descendants` | filter descendants of an element |
| `take_screenshot` | PNG for a `windowHandle`, returned as an image content block |
| `click_element` | click an element by `elementHandle` |
| `drag_element` | drag between elements / points |
| `set_element_value` | set a widget's value (e.g. type into a `LineEdit`) |
| `dispatch_key_event` | key event on a `windowHandle` (`text` + `eventType`); `text:"\n"` = Enter |
| `invoke_accessibility_action` | trigger an a11y action on an element |
| `start_event_recording` / `stop_event_recording` | record a UI interaction |

### Typical loop

```
list_windows {}                                    → windowHandle
get_element_tree {"elementHandle": <root>}         → element handles + ids
set_element_value {"elementHandle": <lineedit>, "value": "20"}
dispatch_key_event {"windowHandle": <win>, "text": "\n", "eventType": "..."}   # commit
get_element_properties {"elementHandle": <lineedit>}   → assert the value
take_screenshot {"windowHandle": <win>}            → eyeball it
```

## Gotchas (learned the hard way)

- **Element handles go stale after a model rebuild.** Any Rust-side `set_ability_fields(...)` (i.e.
  our `refresh`) swaps the `VecModel`, which invalidates every handle under it. Re-fetch the tree
  after any action that triggers a refresh — selecting a party member, committing a field.
- **The element tree stops at 200 elements by default** (`truncated: true`), so with a long list
  on screen later widgets (an open dialog, the last tabs) are missing. Pass `maxElements` (up to
  1000), or use `query_element_descendants` for targeted lookups.
- **Even 1000 elements may not reach a dialog**: modal dialogs come last in the tree, so on a busy
  tab (a long spell list) they fall off the end. Switch to a light tab first, or find the dialog's
  buttons with `query_element_descendants` (`matchElementAccessibleRole: "Button"`, `findAll`).
- **`set_element_value` doesn't focus the input.** To commit with Enter, click the input first,
  then set its value, then `dispatch_key_event` `"\n"`.
- **Clicks land on whatever is on top** at the element's centre — a floating browser window or a
  modal backdrop swallows clicks meant for the editor beneath. Move / close the window first
  (shrink it with its grip, then drag it into a corner).
- **Elements scrolled out of view aren't in the tree** (e.g. the last type checkboxes of a
  browser's filter list): scroll them into view first, or test with one that's visible.
- **`get_element_tree` / `take_screenshot` want the right handle kind.** The tree takes an
  `elementHandle`; the screenshot takes a `windowHandle`. Passing a window where an element is
  expected (or vice-versa) errors.
- **Headless still needs `SLINT_EMIT_DEBUG_INFO=1` at build time** — without it the tree comes back
  without element ids/locations and locating widgets by id fails.
- **Screenshots are true off-screen renders** in headless mode — no window has to be visible or
  focused, unlike the egui/X11 path.
- **Don't `pkill -f infinitier_keeper_slint`** — the pattern matches your own shell command line and
  kills the driving shell. Record the PID (`echo $! > app.pid`) and `kill "$(cat app.pid)"`.
- **A killed app leaves its temp files behind.** Loading a game with compressed (BIFC) archives
  — BG2 does — unpacks each into a `/tmp/infinitier_*` directory (~60 MB) that's removed on
  drop, which `kill` skips. A day of test runs filled the 32 GB RAM-backed `/tmp`. Run test
  instances with a private `TMPDIR` and empty it after each `kill`.
- **`SLINT_EMIT_DEBUG_INFO` is read by the build script**, which doesn't rerun just because the
  variable changed: `touch build.rs` when switching it on for a crate already built without it.
- **Roles come back capitalised** (`Slider`, `ListItem`, `Button`), and zero coordinates are
  left out of `absolutePosition` (read them with a default of 0).
- **The explorer's splitter is a `Slider` too** (it resizes by steps for assistive
  technologies): a real slider is the one with an `accessibleValue`.
- **The explorer's tree is virtualized** — only the rows in view are in the element tree. Move
  through it with the arrow keys (`dispatch_key_event` with `"\uf700"` up, `"\uf701"` down,
  `"\uf702"` left, `"\uf703"` right); the tree has the keyboard from the start.

## Build prerequisite (Linux)

Slint's text stack links `fontconfig` at build time. Install the **dev** package once:

```sh
sudo apt install libfontconfig1-dev     # Debian/Ubuntu
```

The runtime `libfontconfig.so.1` alone is not enough — the `.pc`/`.so` dev symlink is a build-time
linker requirement. See `src/infinitier_keeper_slint/readme.md`.
