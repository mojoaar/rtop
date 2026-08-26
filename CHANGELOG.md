# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- New built-in themes: Dracula, Nord, and GitHub Dark (in addition to the four
  Catppuccin flavors).
- Settings toggle to show or hide the Active/History chart labels in the CPU,
  memory, and GPU panels (default on).

### Changed
- Disk panel now uses a table layout (mount, size, used, free) with the used
  value colored by fullness, instead of a bar with an inlaid percentage.
- Disk and sensors panels now share the row evenly (50/50) instead of 1:2.

## [0.6.0] - 2026-08-25

### Added
- Process filter feedback (and full keybinding list) in the full-screen
  process view footer, so filtering works there with visible feedback.

### Changed
- Reduce idle CPU usage: only redraw the terminal when new data arrives or the
  user acts, instead of every poll tick.
- Cache user accounts and throttle slow-changing reads (temperature sensors,
  GPU, fans, battery) to roughly every 2 seconds instead of every sample.
- Cache per-process CPU time / thread counts (macOS `proc_pidinfo`) across
  samples and refresh them at the throttled cadence, cutting the number of
  per-process syscalls.

## [0.5.0] - 2026-08-25

### Security
- Harden the WAN IP fetch: reject non-http(s) URLs and insert `--` before the
  URL to prevent curl option injection.

### Added
- macOS CI workflow (format, clippy, tests).
- App-routing tests (filter/sort, selection clamping, click mapping) and a
  regression test asserting the filtered, sorted process list is what renders.

### Changed
- Bundle render arguments into view structs and clear all clippy lints.
- Track `Cargo.lock` for reproducible builds.

## [0.4.0] - 2026-08-25

### Added
- Process filter now also matches usernames (in addition to name and PID).
- Full-screen process view (`z`) that expands the process table to the whole
  terminal.
- Settings toggles to show/hide the clock and uptime in the footer (both on by
  default).

### Changed
- Help popup logo replaced with a hand-crafted block-letter `rtop` wordmark.
- Process details popup shows memory with raw KB precision.
- GPU panel title no longer repeats the GPU name (`GPU · GPU`).

### Fixed
- Process details popup now updates CPU, memory, and other stats live while
  open (previously frozen at the time the popup was opened).

## [0.3.0] - 2026-08-25

### Added
- Memory history sparkline in the Memory panel.
- Help popup footer with version, repository link, and author.
- Per-process CPU and memory history graphs in the process details popup.
- Sort direction indicators (↑/↓) in the process table header, with direction
  toggling on repeated sort-key presses.
- Cumulative network totals in the Network panel.
- Per-core CPU rows with usage bars and percentages.
- Load average in the CPU panel title and process count in the Processes title.
- Timezone in the footer clock.
- In-app settings menu (`s`) to change refresh rate, theme, and transparency
  live, with the active sample rate shown in the footer.
- Network panel header shows private and WAN IP addresses.
- WAN IP toggle and editable WAN URL endpoint in the settings menu (fetched
  via `curl`, no added dependencies).
- MIT license and `AGENTS.md` documentation.

### Changed
- Memory and GPU stats text moved to the bottom-right and rendered in muted
  grey (bars keep their fullness color).
- CPU panel: spacing added between the live gauge and the history sparkline.
- Network panel header now uses `prv` for the private address and omits the
  WAN address when WAN IP is disabled.
- Battery state now shows friendly labels (charging / discharging / full /
  empty / on AC).
- Default refresh interval changed to 500 ms.
- Swap row is hidden when no swap is configured.
- Disk usage text moved off the gauge bar for readability.
- Memory and disk panels now use block-bar usage charts (█/░) colored by
  fullness, with high-contrast (non-white) labels.
- Memory stats text now sits below a single-row bar (mirrors the GPU panel).
- Disk and sensors panels are fixed-height; the Processes panel fills the
  remaining space.
- Sensors panel trimmed to battery, CPU/GPU temperature, and fans.
- Help popup ASCII logo is now centered.

## [0.2.0] - 2026-08-25

### Added
- CPU, GPU, and network history sparklines.
- Per-mount disk read/write I/O rates.
- Process filtering by name or PID.
- Process details popup (Enter) and help popup (?).
- Mouse support (click to select, scroll to move).
- CPU model, frequency, and load average display.
- Transparent background option (honors the terminal theme).
- Real process CPU time via `proc_pidinfo` on macOS.
- Footer clock and system uptime.

### Changed
- Default refresh interval changed to 1000 ms.
- Process table columns: PID, User, Name, CPU%, Memory, CPU time, Threads.
- Network panel redesigned as download/upload graphs.
- Disk panel uses colored usage gauges.
- Memory and GPU panels combined into a two-column row.

### Fixed
- Network rates always showing zero (delta vs. cumulative counters).

## [0.1.0] - 2026-08-25

### Added
- Initial release: a bpytop-style terminal system monitor for macOS.
- CPU, memory, swap, processes, network, disk, battery, and sensor panels.
- GPU and fan statistics via IOKit and SMC.
- Catppuccin themes (latte, frappé, macchiato, mocha) with runtime cycling.
- TOML configuration file and CLI overrides (`--theme`, `--interval`).
- Process table with sorting and kill (SIGTERM).
