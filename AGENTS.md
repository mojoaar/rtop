# AGENTS.md

## Project

`rtop` is a terminal system monitor in the style of `top`/`bpytop`, written in
Rust with [ratatui] + [crossterm]. It renders a live dashboard of CPU, memory
& swap, GPU, network, disk, battery & sensors, and a sortable/filterable
process table, themed with seven built-in themes (Catppuccin ×4, Dracula, Nord,
GitHub Dark). macOS is the reference platform; Linux has full GPU (NVML/sysfs)
and fan (hwmon) support; Windows is partially supported through the `sysinfo`
backend.

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
event.rs                crossterm input → Action; Mode { Normal, Filtering, Settings, SettingsEdit, Signal }
theme/                  Theme + ThemeColors semantic roles; catppuccin.rs (4 flavors), presets.rs (Dracula/Nord/GitHub Dark)
data/                   snapshot.rs (types), sysinfo_impl.rs, battery_impl.rs,
                        history.rs (RingBuffer + History + ProcessHistory),
                        rate.rs, format.rs, ip.rs (private/WAN IP monitor)
platform/               macOS FFI: gpu_macos.rs (IOKit), fan_macos.rs (SMC),
                        cpu_time.rs (proc_pidinfo); Linux: gpu_linux.rs (NVML/sysfs),
                        fan_linux.rs (hwmon); signal.rs (Term/Kill/Interrupt)
ui/                     layout + widgets/ (cpu, memory, gpu, network, disk, sensors,
                        processes) + popups.rs (detail/help/settings/signal/net modals)
```

## Hard constraints

- Edition 2021 (not 2024).
- No `unsafe` outside `platform/`.
- Never commit red: run `cargo fmt --check`, `cargo clippy --all-targets -- -D
  warnings`, `cargo test`, and `cargo build` and confirm all are green (0
  warnings, 0 failures) before every commit.
- Graceful degradation: missing hardware (battery/GPU/fans) renders as `n/a`
  or is omitted — never panic.

## Platform notes (macOS reference)

- GPU: `platform/gpu_macos.rs` reads IOKit `PerformanceStatistics` via
  `core-foundation` + raw `extern "C"` IOKit FFI (Apple Silicon `AGXAccelerator`,
  Intel `IOAccelerator`).
- Fans: `platform/fan_macos.rs` reads SMC via `smc-lib`.
- CPU time / threads: `platform/cpu_time.rs` calls `libc::proc_pidinfo` with
  `PROC_PIDTASKINFO` (sums `pti_total_user` + `pti_total_system`).
- Process signals: `platform/signal.rs` uses sysinfo `kill_with` on Unix,
  exposing `SignalChoice::{Term, Kill, Interrupt}` via the `k` signal menu.

## Platform notes (Linux)

- GPU: `platform/gpu_linux.rs` reads NVIDIA via `nvml-wrapper` (`Nvml::init`,
  `utilization_rates`, `memory_info`), falling back to the DRM sysfs interface
  (`/sys/class/drm/card*/device`) for AMD (`gpu_busy_percent`) and Intel
  (`gt_cur_freq_mhz`).
- Fans: `platform/fan_linux.rs` reads `/sys/class/hwmon/hwmon*/fan*_input`.
- `core-foundation` + `smc-lib` are gated to `cfg(target_os = "macos")`;
  `nvml-wrapper` is gated to `cfg(target_os = "linux")`.

## Versioning & changelog

- Semantic versioning; `CHANGELOG.md` follows Keep a Changelog. Bump
  `Cargo.toml` `version` and move `## [Unreleased]` → a dated release section
  when cutting a release, then `git tag vX.Y.Z`.
