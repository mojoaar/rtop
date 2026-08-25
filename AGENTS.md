# AGENTS.md

## Project

`rtop` is a terminal system monitor in the style of `top`/`bpytop`, written in
Rust with [ratatui] + [crossterm]. It renders a live dashboard of CPU, memory
& swap, GPU, network, disk, battery & sensors, and a sortable/filterable
process table, themed with Catppuccin. macOS is the reference platform;
Linux/Windows are partially supported through the `sysinfo` backend.

## Commands

```sh
cargo build          # debug build
cargo test           # run the test suite (must be green before committing)
cargo run            # launch the TUI
cargo build --release
```

## Module map

```
main.rs                 CLI entrypoint (clap), delegates to app::run
app.rs                  app state, event loop, sampler/IP wiring
config.rs               TOML config load/save (dirs::config_dir()/rtop/config.toml)
event.rs                crossterm input → Action; Mode { Normal, Filtering, Settings, SettingsEdit }
theme/                  Theme + ThemeColors semantic roles; catppuccin.rs (4 flavors)
data/                   snapshot.rs (types), sysinfo_impl.rs, battery_impl.rs,
                        history.rs (RingBuffer + History + ProcessHistory),
                        rate.rs, format.rs, ip.rs (private/WAN IP monitor)
platform/               macOS FFI: gpu_macos.rs (IOKit), fan_macos.rs (SMC),
                        cpu_time.rs (proc_pidinfo); signal.rs (kill)
ui/                     layout + widgets/ (cpu, memory, gpu, network, disk, sensors, processes)
```

## Hard constraints

- Edition 2021 (not 2024).
- No `unsafe` outside `platform/`.
- Never commit red: run `cargo test` and `cargo build` and confirm both are
  green (0 warnings, 0 failures) before every commit.
- Graceful degradation: missing hardware (battery/GPU/fans) renders as `n/a`
  or is omitted — never panic.

## Platform notes (macOS reference)

- GPU: `platform/gpu_macos.rs` reads IOKit `PerformanceStatistics` via raw
  `core_foundation_sys` FFI (Apple Silicon `AGXAccelerator`, Intel `IOAccelerator`).
- Fans: `platform/fan_macos.rs` reads SMC via `smc-lib`.
- CPU time / threads: `platform/cpu_time.rs` calls `libc::proc_pidinfo` with
  `PROC_PIDTASKINFO` (sums `pti_total_user` + `pti_total_system`).
- Process signals: `platform/signal.rs` uses sysinfo `kill_with` on Unix.

## Versioning & changelog

- Semantic versioning; `CHANGELOG.md` follows Keep a Changelog. Bump
  `Cargo.toml` `version` and move `## [Unreleased]` → a dated release section
  when cutting a release, then `git tag vX.Y.Z`.
