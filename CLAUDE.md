# Infinitier

There are four GUIs over the shared `infinitier_core` data layer, two apps in two toolkits:
the save editor `infinitier_keeper` (egui) and `infinitier_keeper_slint`, and the resource
browser `infinitier_explorer` (egui) and `infinitier_explorer_slint`. The Slint ones are built on
the [slint-themekit](../../github_ufoscout/slint-themekit) component library — no
`std-widgets` — and share no code with their egui twins (logic is duplicated on purpose).

## GUI verification

**egui keeper** — drive it through the **egui MCP**, see [docs/egui_mcp.md](docs/egui_mcp.md).
Run with `EGUI_INSPECTION=1` and `attach`; then `query_tree` for widget ids/values and act by id.

**Slint keeper / explorer** — drive them through the **Slint MCP**, see
[docs/slint_mcp.md](docs/slint_mcp.md).
Run with `--features mcp` and `SLINT_EMIT_DEBUG_INFO=1 SLINT_MCP_PORT=8080 SLINT_BACKEND=headless`
(headless renders frames without a compositor — needed on this Wayland box); then
`get_element_tree` for element handles and act on them. Note element handles go stale after any
model rebuild — re-fetch the tree after selecting a party member or committing a field.
Give test runs a private `TMPDIR` and empty it after killing them: a killed app leaves ~60 MB
per compressed BIF in `/tmp`, which is RAM.
Build prereq on Linux: `libfontconfig1-dev`.

Prefer these over `xdotool` + screenshot coordinate-hunting: no stolen window focus, and you can
assert widget values directly instead of reading pixels.
