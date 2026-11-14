# Local-player checkpoint — 0.2.0 · 14 November 2026

## Delivered

Local preview: `G:\Essential Softwares\MuseFlow-PC\Preview\0.2.0-local\MuseFlow-PC.exe`.

Real local audio through a dynamically loaded libmpv adapter, native file/folder selection, background tag scanning, SQLite library/settings, likes, basic playlists, virtualized queue, seek/volume, shuffle and repeat. Restart restores the queue paused; a data-folder lock protects against concurrent writers. Normal launches use real library data, while `--demo` remains an explicit separate fictional shell.

The preview's normal data location is `G:\Essential Softwares\MuseFlow-PC\Data`. Source/references remain on E:. The old 0.1.0 shell is preserved. No Android changes, Git initialization, publishing, registry/PATH changes, separate FFmpeg CLI or GPU driver installation occurred.

The executable is 17,069,056 bytes; the adjacent libmpv DLL is 121,789,952 bytes. Together they occupy about 132.43 MiB before notices/data. Keep the DLL and portable marker beside the executable. This is an unsigned local preview, not an installer/public release.

## Verification completed

- Optimized release build, Clippy with warnings denied and **14 Rust tests passed**. Tests cover filtering/cancellation/identity; queue order, shuffle/repeat; Unicode/dedup/persistence, foreign keys, future-schema refusal and rollback after injected SQLite failures. Progress-only persistence is checked not to rewrite queue rows.
- Native local journey passed real decoding/time advance, pause, seek, volume, Next/Previous, EOF Repeat one, automatic Repeat all advance, likes/playlists and queue state.
- Restart restored paused position/volume/modes without initializing libmpv. Missing-file Next recovery, missing-runtime error, 30 rapid selections and second-instance refusal passed.
- FLAC, AAC/M4A, MP3 and Opus fixtures reported their actual respective decoders and progressed. WAV PCM was used for main journeys. Other accepted scan extensions are not all fixture-certified.
- A real Windows file-picker journey imported a generated Bengali-named WAV, reported correct counts and started playback from the visible row. No personal music files were imported or altered.
- The packaged executable passed the local journey with developer runtime/data overrides removed, using its adjacent DLL and isolated test data.
- The preserved 10,000-record demo regression passed with both FemtoVG and software renderers. Actual Slint-rendered local/queue screenshots were inspected.
- Silent PCM opened the default Windows **WASAPI** endpoint and advanced at 48 kHz. Main application journeys use null output; this does not certify audible sound quality, headphones or device-switching behavior.
- Redacted Gitleaks scan found no leaks using the existing exact-path/value public InnerTube constant exception. All **252** copied references and their Android originals still match preparation hashes.

## Measurements on this PC

Windows 11 Pro 10.0.26300, Ryzen 5 7500F / RTX 4070, optimized binary, FemtoVG. Three warm cached empty-library launches; first-frame marker requires library-ready and AfterRendering. Observed times poll at 10 ms. Idle samples taken after three seconds. One process, no resolver/helper process tree to add.

| Measurement | Result |
| --- | --- |
| Observed warm first frame, median | 256.23 ms (range 244.23–286.54 ms) |
| Main-to-render samples | 239, 267, 231 ms |
| Idle working set, median | 88.69 MiB |
| Idle private bytes | 95.86–96.69 MiB |
| Short null-output playback journey, peak process working set | 125.15 MiB |
| Same journey, maximum sampled private bytes | 146.02 MiB |

Warm startup and idle working-set targets are met under these conditions. The short playback journey is not a long-session steady real-output measurement. Reboot-cold startup, GPU memory, frame-time distributions, real large-library scrolling/scanning, long-session growth and streaming remain unmeasured. Historical shell-only measurements are preserved in SHELL-CHECKPOINT.md and must not be substituted for these figures.

## Remaining limits

Embedded album art currently uses a neutral placeholder. Playlist rename/delete/reorder and library removal are not implemented. Online catalog/resolution, lyrics lookup, media keys, tray, downloads, DSP and installers remain later milestones.

Windows 10, clean-machine installation, audio-device loss/switch, full screen-reader/keyboard/scaling acceptance and malformed-media stress require further validation. An initial computer-use capture of the previously occluded window appeared blank before interaction; later frames and both renderer journeys worked. Capture versus rendering cause is unresolved; occlusion/resume should be reproduced before closing that observation.

The selected libmpv's import table includes **vulkan-1.dll**, present through this PC's graphics runtime. It is not a core-Windows guarantee and is not bundled here. No separately installed Rust/MinGW runtime is needed. Clean-machine compatibility must include this dependency or use a separately validated audio-only runtime build. Full transitive binary-license notices/corresponding-source fulfillment also remain a public-distribution prerequisite.

## Evidence

`E:\ALL Project & Idea Workspace\WSL BACKUP\artifacts\museflow-pc-local-player-2026-11-14` holds logs, dependency inventory, provenance, hashes, generated fixtures, native snapshots and performance data. Current successful native checkpoints use `verified-*` paths. Earlier failed harness checkpoints are preserved as diagnostic history; a test originally stalled in a one-track playlist, and a performance rerun originally reused a liked-state database. Fresh isolated data corrected those harness preconditions; neither old result is counted as a pass.

EXE SHA-256: `8bf13bbe8d6c2d8a7ebe5f0c1dffe2d97ff371259d7dc6be8de7e9ea481e3c06`.

libmpv SHA-256: `88504ee5560942a955405e99dfb5971368e4cf851321f36565bf2a667271db1b`.
