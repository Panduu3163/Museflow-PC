# Desktop shell checkpoint — 14 November 2026

## Delivered

Native Windows executable: `G:\Essential Softwares\MuseFlow-PC\Preview\0.1.0-shell\MuseFlow-PC-Preview.exe` (13,347,328 bytes, about 12.73 MiB). This is an unsigned local shell preview, not a packaged release or a working audio player.

Rust 1.100.0 and Slint 1.18.1 provide Home, Search, Library, Liked songs, Settings, dark/light themes, a track-details/lyrics placeholder panel and persistent player preview. The list uses a lazy model over 10,000 synthetic metadata records; filtering runs on a background worker with one pending slot, cancellation and stale-result guards. Likes/theme are session-only. No personal library, Android database, account or network source is accessed.

Playback buttons are deliberately disabled and the sample-data banner is visible. Actual audio, SQLite persistence, file import, online search/streaming, queue management, lyrics lookup and Windows media controls remain later milestones. No elaborate glass effects were introduced.

## Checks completed

- Optimized release build succeeded; Rust dependency versions are locked.
- Four Rust regression tests passed: stable identity through filtering, liked filtering/order, cancellation without partial results and bounded Home/full Library/empty-result recovery.
- Clippy passed with warnings denied; formatting applied to the new Rust source only.
- Native window smoke journeys passed with both **winit-femtovg** and **winit-software**. Pointer and Enter-key events activate Library; controller journeys check 10,000 rows, filtering, empty results, stable selection IDs, unlike, theme change and a narrow window. These are not a complete physical mouse/keyboard or screen-reader certification.
- Actual rendered dark/light/narrow screens were inspected. Corrected the initial maximum-width constraint, expanding header controls and short-panel clipping; narrow details now scroll with a reachable close button.
- The preview executable was launched without the compiler environment. Its import table requires Windows system DLLs, not a separately installed Rust compiler or MinGW runtime. This is local-host evidence, not a clean-machine installation test.
- Secret scan passed; original copied reference hashes and current Android source hashes still match all 252 copied entries. No Android implementation changes, Git initialization or publication occurred.

## Measurements on this PC

Windows 11 Pro 10.0.26300; Ryzen 5 7500F / RTX 4070; optimized build; FemtoVG; 10,000 synthetic metadata records; three warm launches. Ready-file observation polls at 10 ms; idle memory sampled after three seconds.

| Measurement | Result |
| --- | --- |
| Observed first rendered frame, median | 255.16 ms (224.47–255.89 ms) |
| Internal main-to-first-render samples | 210–241 ms |
| Idle working set, median | 90.00 MiB |
| Private bytes | 101.29–103.00 MiB |
| Search-computation microbenchmark | 100 filters over 10,000 records in 6.06 ms |
| Resolver/runtime helper processes | None started by this shell |

The shell's observed warm-start/idle targets are met on this host. GPU memory, reboot-cold startup, playback memory, scrolling frame-time distributions, large-text/200% scaling, long-session memory growth, Windows 10 and complete accessibility acceptance remain pending. The filter microbenchmark is not an end-to-end search latency or scrolling benchmark. Do not extrapolate these figures to the finished music player.

## Evidence and next stage

Logs, actual rendered PNGs, dependency/license inventory, redacted secret scan and `performance.json` are in `E:\ALL Project & Idea Workspace\WSL BACKUP\artifacts\museflow-pc-shell-2026-11-14`. The verified renderer folders start with `verified-`; earlier captures in that artifact directory describe layouts corrected during development.

Executable SHA-256: `7e814536138806aeee023dfabe719dd83daa20985a9166239cc04a01e131b52d`.

Next implementation stage is the local libmpv playback adapter, desktop file selection/metadata scanning, SQLite library and durable queue. It needs real small codec fixtures and explicit error/recovery tests before enabling playback controls. Online resolution follows local playback. Android development and build-66 phone investigation remain paused.
