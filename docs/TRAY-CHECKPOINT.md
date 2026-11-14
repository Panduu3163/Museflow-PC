# Tray/portable checkpoint — 0.5.0 · 14 November 2026

## Delivered

Run `G:\Essential Softwares\MuseFlow-PC\Preview\0.5.0-tray\MuseFlow-PC.exe`.

The Windows tray now provides Show, Play/Pause, Previous/Next and explicit Exit. Closing exits by default; Settings adds a saved Close window to tray opt-in and explicit Hide/Exit actions. Hiding retains the native HWND, audio playback and media session. Show restores hidden/minimized windows and requests a new frame. Lyrics requests are suspended while hidden/minimized.

Hiding is gated by real tray availability, not just successful object construction. Failed tray creation leaves Close as Exit. A hidden window restores if the tray becomes unavailable. The dependency handles taskbar-recreated messages; an actual Explorer restart was not performed.

A hidden fast Pause→Play test exposed a Windows status-publication race. The bridge now synchronizes immediately after core state changes; unchanged metadata/progress are not republished. Repeated hidden media commands passed afterward. This is a specific race fix, not a guarantee covering every earlier provider/stream issue.

Windows file/product version resources derive from Cargo.toml. The portable packager checks that version and x64 architecture, verifies staged binary checksums, includes an explicit allowlist of binaries/notices/licenses, excludes data/caches/source/keys, writes a relative data marker and content manifest, and verifies archive CRC/content hashes. Output has its own SHA-256. It refuses an existing output file and performs no installation/signing/publication.

The local preview keeps existing data at `G:\Essential Softwares\MuseFlow-PC\Data`; an extracted portable copy uses its adjacent data/ folder. Source/work remains on E:. Toolchains, dependencies, caches, build output, distribution and extraction tests stay in separate G: folders. SQLite remains schema 2. No Android changes, Git initialization/publishing, autostart or global registry/PATH changes occurred.

## Verified

- Release build, formatting and Clippy with warnings denied; **29 Rust tests passed**. Two ignored entries are child-process fixtures invoked by the tested Windows job teardown, not outstanding product tests.
- Four packaged native journeys passed: opt-in native Close hides while decoding continues; Show restores hidden/minimized states; default Close exits; Exit stops a hidden player through normal cleanup; saved preference/paused queue restore; simulated tray failure cannot strand an opted-in window. SQL inspection confirmed the saved preference values.
- Real Windows media-session inspection and repeated fast Pause→Play, Previous/Next and metadata operations passed with the player hidden. The test targeted only this app's generated fixture session.
- Packaged local regression passed decoding, seek/volume, Next/Previous, EOF repeat/advance and library/playlist/queue behavior. The 10,000-record demo regression passed on both FemtoVG and software renderers.
- The packaged live online journey passed exact recording search/resolution, decode/time progression and a settled paused seek with developer runtime overrides removed. No tracked helpers remained and normal helper scratch cleanup completed.
- Visible computer-use check passed Settings opt-in and title-bar Close hiding. The tool did not expose Windows' notification area, so actual icon/menu mouse interaction remains **pending**. Native shared-action lifecycle checks do not replace that mouse test. The isolated manual-test process was deliberately cleaned up after executable identity verification.
- Portable candidate extraction/launch passed in a G:-hosted path containing spaces and Bengali characters. The relative data marker created its own data folder; content hashes, version/x64 checks, no-data inclusion and a settled live online playback/seek journey passed without developer runtime overrides.
- All **252** copied references and their Android originals still match preparation hashes. Secret scanning uses the existing narrow reviewed public InnerTube constant exception. Final redacted scan result is recorded with the artifacts.

## Measurements on this PC

Windows 11 Pro 10.0.26300, Ryzen 5 7500F / RTX 4070, packaged optimized build, FemtoVG. Three warm cached empty-library launches; first-frame marker requires library ready and AfterRendering, external polling at 10 ms. Idle sampled after three seconds. Benchmark processes now exit normally using an opt-in auto-exit timer, removing their tray icons. Short null-output online journey samples the full descendant process tree every 50 ms.

| Measurement | Result |
| --- | --- |
| Warm first frame, median | 287.44 ms (257.05–338.77 ms) |
| Idle working set, median | 96.00 MiB |
| Main process peak working set in online journey | 158.10 MiB |
| Temporary summed resolver-tree working-set peak | 474.32 MiB |
| Temporary summed private-bytes peak | 425.68 MiB |
| Tracked helpers remaining after journey | None |

Warm/idle targets are met under these conditions. Transient resolver RAM remains substantial and is included above. Sampling can miss shorter peaks; individual process peaks are not simultaneous totals. Reboot-cold startup, GPU memory, long-session steady real-output playback and frame-time distributions remain unmeasured. Main binaries total 251.62 MiB before notices/data; ZIP compressed size/hash are recorded separately.

## Limits and next work

Physical media keys/headphones/Bluetooth, audio-device switching/loss, complete accessibility/scaling, actual tray menu mouse interaction, Explorer restart, Windows 10 and clean-machine tests remain pending. Reopening an already active library still reports it in use; use Show on the existing tray icon. Signed installer, Start-menu integration, artwork and advanced library features remain later work.

The earlier 0.4 intermittent streamed restart observation was not conclusively assigned a cause. A separate status race is now fixed; provider restrictions/network changes and exhaustive expired-URL recovery remain limits. Lyrics scroll manually; OS timeline scrubbing is not wired by the current Windows backend. The libmpv Vulkan-loader dependency and complete native/runtime license/source-fulfillment review remain public-release prerequisites. This is an unsigned local portable preview, not a bug-free/certified release or a clean-machine installer.

## Evidence

`E:\ALL Project & Idea Workspace\WSL BACKUP\artifacts\museflow-pc-tray-2026-11-14` contains source backup, build/unit/native logs, generated-fixture Windows-session commands, tray settings screenshot, manual-check notes, measurements, package metadata, reference/secret checks and archive verification. Earlier failed hidden-media observation is diagnostic history; final native logs determine reported passes.

EXE SHA-256: `5b98151851c75962ccade34d5379e01c1ccc1ffb8a01238c6b5a9f69dccb4c0e`. Packaged binary hashes are in SHA256SUMS.txt; the archive has a separate checksum.
