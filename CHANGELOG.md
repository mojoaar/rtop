# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
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

### Changed
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
