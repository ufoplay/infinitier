# Infinitier

There are two GUIs over the shared `infinitier_core` data layer:
`infinitier_keeper` (egui) and `infinitier_keeper_slint` (Slint, vanilla `std-widgets`).

## GUI verification

**egui keeper** — drive it through the **egui MCP**, see [docs/egui_mcp.md](docs/egui_mcp.md).
Run with `EGUI_INSPECTION=1` and `attach`; then `query_tree` for widget ids/values and act by id.

**Slint keeper** — drive it through the **Slint MCP**, see [docs/slint_mcp.md](docs/slint_mcp.md).
Run with `--features mcp` and `SLINT_EMIT_DEBUG_INFO=1 SLINT_MCP_PORT=8080 SLINT_BACKEND=headless`
(headless renders frames without a compositor — needed on this Wayland box); then
`get_element_tree` for element handles and act on them. Note element handles go stale after any
model rebuild — re-fetch the tree after selecting a party member or committing a field.
Build prereq on Linux: `libfontconfig1-dev`.

Prefer these over `xdotool` + screenshot coordinate-hunting: no stolen window focus, and you can
assert widget values directly instead of reading pixels.
