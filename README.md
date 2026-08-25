# rtop

A beautiful terminal system monitor in the style of `top` and `bpytop`, written in
Rust with [ratatui] + [crossterm]. It renders a live dashboard of CPU, memory,
GPU, network, disk, battery & sensors, and a sortable process table — all themed
with [Catppuccin].

[ratatui]: https://ratatui.rs
[crossterm]: https://github.com/crossterm-rs/crossterm

## Features

- Live CPU (global + per-core), memory & swap, GPU, network I/O rates (with
  private/WAN IP in the header), and disk usage
- Battery status, temperature sensors, and fan speeds (where available)
- Sortable, filterable process table with selection, mouse support, per-process
  details popup (with CPU/memory history graphs), and process kill
- Four Catppuccin flavors with runtime theme cycling and optional transparent background
- Sparkline history for CPU, GPU, and network; per-core CPU load, model, and load average
- Block-bar usage charts for memory and disk (colored by fullness)
- In-app settings menu to change refresh rate, theme, transparency, and WAN IP
  live
- Config file for theme and refresh interval, overridable via CLI flags
- Non-blocking data collection on a background thread — a slow sensor never janks the UI

## Install

```sh
cargo install --path .
```

## Usage

Run from the repository:

```sh
cargo run
```

Or, once installed:

```sh
rtop
```

CLI flags override config values:

```sh
rtop --theme macchiato --interval 500
```

| Flag          | Description                                    |
| ------------- | ---------------------------------------------- |
| `--theme`     | Catppuccin flavor: `latte`, `frappe`, `macchiato`, `mocha` |
| `--interval`  | Refresh interval in milliseconds               |

## Keybindings

| Key             | Action                                  |
| --------------- | --------------------------------------- |
| `q`             | Quit                                    |
| `t`             | Cycle Catppuccin theme                  |
| `c` / `m` / `p` / `n` | Sort processes by CPU / memory / PID / name |
| `↑` / `↓`       | Move process selection                  |
| `k`             | Kill the selected process               |
| `f`             | Filter processes by name or PID         |
| `s`             | Open the settings menu                  |
| `Enter`         | Show details for the selected process   |
| `?`             | Show the help popup                     |
| `Esc`           | Close popup / cancel filter             |
| Mouse click     | Select the process under the cursor     |
| Mouse scroll    | Move process selection                  |

While filtering, type to match process names (case-insensitive) or PIDs, `Enter`
to apply, `Esc` to cancel, `Backspace` to delete.

## Themes

`rtop` ships with all four Catppuccin flavors:

- `latte`
- `frappe`
- `macchiato`
- `mocha`

Select one at startup with `--theme <flavor>`, or press `t` at any time to cycle
through them. The chosen flavor is persisted to the config file.

## Settings menu

Press `s` to open the in-app settings menu. Use `↑`/`↓` to pick a row and
`←`/`→` to change its value; `Esc` closes it. Changes apply immediately and
are persisted to the config file:

| Setting     | Values                                            |
| ----------- | ------------------------------------------------- |
| Refresh     | `100`, `250`, `500`, `1000`, `2000`, `5000` ms    |
| Theme       | `latte`, `frappe`, `macchiato`, `mocha`           |
| Transparent | `on` / `off` (terminal background vs Catppuccin)  |
| WAN IP      | `on` / `off` (fetch public IP via the URL below)  |
| WAN URL     | endpoint returning your public IP (press `Enter` to edit) |

The active refresh rate is shown in the footer next to the clock and uptime.
When WAN IP is enabled, the Network panel header shows `private` and `wan`
addresses (fetched with `curl` from the configured URL).

## Config

Settings live in a TOML file at the platform config directory (via the `dirs` crate):

| Platform | Path                                        |
| -------- | ------------------------------------------- |
| macOS    | `~/Library/Application Support/rtop/config.toml` |
| Linux    | `~/.config/rtop/config.toml`                |
| Windows  | `%APPDATA%\rtop\config.toml`                |

Format:

```toml
[theme]
flavor = "mocha"        # latte | frappe | macchiato | mocha

[general]
interval_ms = 500       # sampling + render tick, in milliseconds
transparent = true      # use the terminal's background (false = Catppuccin bg)
wan_enabled = false     # fetch and display the public (WAN) IP
wan_url = "https://echo.johansen.foo/api/ip"  # endpoint returning your public IP
```

The file is created automatically when you cycle themes (`t`).

## Platform support

| Feature                        | macOS | Linux | Windows |
| ------------------------------ | :---: | :---: | :-----: |
| CPU, memory & swap             |  ✓    |  ✓    |   ✓     |
| Network & disk I/O             |  ✓    |  ✓    |   ✓     |
| Processes (list/sort/kill)     |  ✓    |  ✓    |   ✓*    |
| Battery                        |  ✓    |  ✓    |   ✓     |
| Temperatures                   |  ✓    |  ✓*   |   ✓*    |
| GPU utilization & memory       |  ✓    |  —    |   —     |
| Fan speed (SMC)                |  ✓    |  —    |   —     |

`*` — provided by the cross-platform `sysinfo` backend; platform-specific
behavior may vary.

**macOS** is the reference platform and is fully supported, including GPU
(IOKit `PerformanceStatistics`) and fan speed (SMC via `smc-lib`).

**Linux** and **Windows** are partially supported: CPU, memory, network, disk,
and processes work through `sysinfo`, but GPU and fan metrics are macOS-only for
now and render as `n/a`. Missing sensors degrade gracefully rather than crashing.
