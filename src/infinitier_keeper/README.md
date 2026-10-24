# Infinitier Keeper

A save-game editor for the Infinity Engine games. Inspired by ShadowKeeper,
EEKeeper and JSEEKeeper, but one binary works across every engine, and all
game-specific knowledge (names, tables, limits) is read from your installed
game rather than hard-coded.

![Editing a character](screenshots/editor.png)

## Supported games

| Game | Expansions / variants |
| --- | --- |
| Baldur's Gate | Tales of the Sword Coast |
| Baldur's Gate: Enhanced Edition | Siege of Dragonspear |
| Baldur's Gate II | Shadows of Amn, Throne of Bhaal |
| Baldur's Gate II: Enhanced Edition | |
| Icewind Dale | Heart of Winter, Trials of the Luremaster |
| Icewind Dale: Enhanced Edition | |
| Icewind Dale II | |
| Planescape: Torment | |
| Planescape: Torment: Enhanced Edition | |

The Enhanced Edition Trilogy (EET) and BG1 Tutu mods are recognised too.
What each editor tab can change differs per game; see
[What you can edit](#what-you-can-edit).

## Platforms

Pre-built binaries for **Linux** (x86_64), **Windows** (x86_64) and **macOS**
(Apple Silicon and Intel) are attached to each
[GitHub release](https://github.com/ufoscout/infinitier/releases).
Development and testing happen on Linux.

To build from source (requirements in the [root README](../../README.md#building)):

```sh
cargo run --release -p infinitier_keeper
```

## Configuration

The keeper keeps its settings in `infinitier_keeper.toml` in the folder it is
started from. Pass `--config <file>` to use another file.

Each game has:

- **`game_path`** — the install folder, the one containing `CHITIN.KEY`.
- **`save_path`** (optional) — the folder containing the `save/` directory,
  when it isn't inside the install; see [where each game keeps it](#1-add-a-game).
- **`game_type`** (optional) — forces the game type instead of detecting it
  from the install. Pick it in the Settings dialog.

`--log <filter>` sets the log level, e.g. `--log warn`.

## Add a game

Click **Settings** in the top-right corner. Under **Add a game**, enter the
game folder and, for an Enhanced Edition, its savegame folder, then click
**Add**. Leave **Game type** on *Autodetect* unless detection gets it wrong;
the list shows the detected type next to each game.

**Why a separate savegame folder?** The original games write their saves into
a `save/` folder inside the install folder, so the game path is all the keeper
needs. The Enhanced Editions instead keep saves (with portraits and other
per-user files) in a folder in your user profile, away from the install. Point
**Savegame path** at that folder (the one *containing* `save/`, not `save/`
itself), or the keeper finds no saves to load.

| Game | Windows | Linux |
| --- | --- | --- |
| Baldur's Gate, Baldur's Gate II, Icewind Dale, Icewind Dale II, Planescape: Torment | *leave empty* (`<install>\save`) | *leave empty* (`<install>/save`) |
| Baldur's Gate: EE (and Siege of Dragonspear) | `%USERPROFILE%\Documents\Baldur's Gate - Enhanced Edition` | `~/.local/share/Baldur's Gate - Enhanced Edition` |
| Baldur's Gate II: EE (and EET) | `%USERPROFILE%\Documents\Baldur's Gate II - Enhanced Edition` | `~/.local/share/Baldur's Gate II - Enhanced Edition` |
| Icewind Dale: EE | `%USERPROFILE%\Documents\Icewind Dale - Enhanced Edition` | `~/.local/share/Icewind Dale - Enhanced Edition` |
| Planescape: Torment: EE | `%USERPROFILE%\Documents\Planescape Torment - Enhanced Edition` | `~/.local/share/Planescape Torment - Enhanced Edition` |

- **Running a Windows version under Wine on Linux:** the Enhanced Editions use
  the Windows path inside the Wine prefix, e.g.
  `~/.wine/drive_c/users/<you>/Documents/Baldur's Gate - Enhanced Edition`.
- **An original game installed under `Program Files`:** Windows may redirect
  its saves to
  `%LOCALAPPDATA%\VirtualStore\Program Files (x86)\<install folder>`; if the
  install's `save\` folder is empty, use that folder as the savegame path.

![Adding a game in the Settings dialog](screenshots/settings.png)
