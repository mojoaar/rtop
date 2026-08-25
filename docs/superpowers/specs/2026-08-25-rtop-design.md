# rtop — Terminal System Monitor (Design Spec)

Date: 2026-08-25

## Overview

`rtop` is a terminal UI application that reads and displays live system status, in the
style of `top` and `bpytop`. It is written in Rust and must run on macOS, Linux, and
Windows. The first milestone targets macOS; Linux and Windows are ported later.

The TUI is built with **ratatui** (v0.30) + **crossterm** (its default cross-platform
backend). The visual design is a bpytop-style full dashboard, themed, with **Catppuccin**
as the first theme.

## Goals

- Render a beautiful, live system dashboard: CPU, memory & swap, GPU, network, disk,
  battery & sensors, and a process table.
- Allow full process control: sort the table and send signals (kill) to selected processes.
- Support themes, starting with Catppuccin (4 flavors), with runtime flavor cycling and a
  path to user-defined themes.
- Persist settings (theme, interval) in a platform config file, overridable via CLI flags.
- Keep data collection responsive (never block the UI) and resilient (missing sensors
  degrade gracefully rather than crash).
- Cleanly separate cross-platform code from macOS-specific code so Linux/Windows porting is
  incremental.

## Non-Goals (v1)

- Linux and Windows data backends (deferred; only the cross-platform `sysinfo`/`battery`
  path is shared).
- GPU stats on Linux/Windows.
- Custom (user-authored) theme files — the `Theme` type is `serde`-ready for this, but only
  Catppuccin flavors ship in v1.
- Daemon / client-server architecture.

## Architecture

Layered, single binary crate:

```
rtop/
├── Cargo.toml
├── src/
│   ├── main.rs            # terminal setup, teardown guard, launch loop
│   ├── app.rs             # App state, tick loop, input dispatch
│   ├── event.rs           # crossterm event handling (key/mouse/resize)
│   ├── config.rs          # Config + Theme loading/saving (TOML), CLI merge
│   ├── theme/
│   │   ├── mod.rs         # Theme struct + semantic roles + registry
│   │   └── catppuccin.rs  # 4 flavors (Latte/Frappé/Macchiato/Mocha)
│   ├── data/
│   │   ├── mod.rs         # MetricsProvider trait + sampling thread
│   │   ├── snapshot.rs    # typed snapshot types
│   │   ├── history.rs     # ring buffer for graphs
│   │   ├── sysinfo_impl.rs# default cross-platform provider
│   │   └── battery_impl.rs
│   ├── platform/
│   │   ├── mod.rs         # ProcessControl + GpuStats + FanStats traits
│   │   ├── gpu_macos.rs   # IOKit PerformanceStatistics
│   │   ├── fan_macos.rs   # SMC fan speed (smc-lib)
│   │   └── signal_unix.rs # kill/kill_with(Signal)
│   └── ui/
│       ├── mod.rs         # root render + layout
│       └── widgets/       # cpu, mem, gpu, net, disk, battery, sensors, proc
```

### Data flow

A background thread samples system state every `interval_ms` (default 250 ms):

1. Refresh `sysinfo` and `battery`.
2. Compute rate deltas for network and disk I/O (sample twice and diff; `sysinfo` exposes
   cumulative byte counters).
3. Read GPU utilization/memory (macOS IOKit) and fan speed (macOS SMC).
4. Build a `Snapshot` and push it into a `History` ring buffer (~120 samples ≈ 30 s).

The UI loop polls `crossterm` events and renders the latest snapshot non-blocking. A slow
`sysinfo` refresh never janks the UI because collection happens off the render thread.

### Isolation

Each unit has one clear purpose and a well-defined interface:

- `MetricsProvider` trait — produces a `Snapshot`; one default cross-platform impl, plus
  macOS additions (GPU, fan). UI depends only on `Snapshot`, not on `sysinfo`.
- `Snapshot` / `History` — plain data, no rendering, unit-testable in isolation.
- `Theme` — maps semantic color roles to ratatui `Color`; rendering never hardcodes a hex.
- `platform` traits (`ProcessControl`, `GpuStats`, `FanStats`) — isolate OS-specific code;
  each is `cfg`-gated per target and returns `Option`/empty on unsupported systems.

## Data collection

| Metric          | Source (v1, macOS)                                  | Notes                                              |
| --------------- | --------------------------------------------------- | -------------------------------------------------- |
| CPU             | `sysinfo` per-core + global                         | cross-platform                                     |
| Memory / swap   | `sysinfo`                                           | cross-platform                                     |
| Processes       | `sysinfo` (incl. `kill()` / `kill_with(Signal)`)    | `kill_with` unix-only; Windows degrades to `kill()` |
| Network rates   | `sysinfo` cumulative counters, diffed               | cross-platform                                     |
| Disk usage/I-O  | `sysinfo` disks                                      | cross-platform                                     |
| Temperatures    | `sysinfo` `Components` (SMC keys on x86, HID on AS) | cross-platform path, macOS verified                |
| Battery         | `battery` crate                                      | cross-platform                                     |
| Fan speed       | `smc-lib` (macOS SMC)                                | macOS-only                                         |
| GPU util/memory | IOKit `PerformanceStatistics` (see below)            | macOS-only in v1                                   |

### GPU (macOS)

Read IOKit `PerformanceStatistics` from `AGXAccelerator` (Apple Silicon) or `IOAccelerator`
(Intel) via `objc2-io-kit`:

- `IOServiceMatching(class)` then `IORegistryEntryCreateCFProperty(service,
  "PerformanceStatistics")`.
- Extract `"Device Utilization %"`, `"Renderer Utilization %"`, `"In use system memory"`,
  `"Alloc system memory"`.

No sudo, no private API. On Apple Silicon, GPU memory is unified memory. Linux (NVML/sysfs)
and Windows (PDH/DXGI) GPU backends are deferred.

## Theme system

A `Theme` struct maps **semantic roles** to `ratatui::style::Color`:

`bg, fg, text, muted, accent, success, warning, danger, info, surface, border, highlight`

Catppuccin is implemented by mapping the `catppuccin` crate's 4 flavors (Latte, Frappé,
Macchiato, Mocha) onto these roles (Mocha: base `#1e1e2e` bg, `#a6e3a1` success,
`#f38ba8` danger, `#89b4fa` accent, etc.). Runtime flavor cycling via a key binding.
`Theme` derives `serde` so custom themes can be loaded from TOML later without changing
rendering code.

## Config file

TOML at the platform config dir (via the `dirs` crate):

- macOS: `~/Library/Application Support/rtop/config.toml`
- Linux: `~/.config/rtop/config.toml`
- Windows: `%APPDATA%\rtop\config.toml`

```toml
[theme]
flavor = "mocha"        # latte | frappe | macchiato | mocha

[general]
interval_ms = 250       # sampling + render tick
```

CLI flags (`--theme`, `--interval`) override config values.

## Layout (top to bottom)

1. **CPU** — overall % + per-core mini bars + history sparkline.
2. **Memory & swap** — used/free bar + swap bar.
3. **GPU** — utilization % + GPU memory + history.
4. **Network** — up/down rates + sparklines.
5. **Disk** — per-mount usage + read/write I/O.
6. **Battery & sensors** — battery %/state, temperature list, fan RPM.
7. **Processes** — sortable table (PID, name, CPU%, mem%, state) + selection + signal menu.

## Error handling & resilience

- Data sources degrade gracefully: missing battery (desktop), SMC/IOKit permission denied,
  etc. render `n/a` instead of crashing.
- `anyhow` for top-level failures; `thiserror` for library error types.
- A `Drop` guard guarantees terminal raw-mode restore on panic or exit.

## Dependencies

`ratatui`, `crossterm`, `sysinfo`, `battery`, `smc-lib`, `objc2-io-kit`, `catppuccin`
(ratatui feature), `clap`, `serde`, `toml`, `dirs`, `anyhow`, `thiserror`, `unicode-width`.

## Testing

- Unit tests for pure logic: rate calculation, history ring buffer, theme mapping, process
  sorting, byte/percent formatting.
- Widget render snapshots via ratatui `TestBackend`.
- Manual verification on macOS for the IOKit/SMC/battery paths.

## Milestones

1. Scaffold + terminal setup + empty loop with exit key.
2. Config system (load/save TOML, CLI overrides).
3. Theme system + Catppuccin 4 flavors + runtime cycling.
4. Data layer (sampling thread, snapshot, history ring buffer).
5. CPU / memory / network / disk widgets.
6. Battery + sensors + fan (SMC).
7. GPU panel (IOKit PerformanceStatistics).
8. Process table + sort + signals.
9. Polish, README, performance pass.
10. *(later)* Linux + Windows porting (incl. their GPU backends).
