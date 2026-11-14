# Playback recovery checkpoint — 0.5.1 · 15 November 2026

## Result

Run `G:\Essential Softwares\MuseFlow-PC\Preview\0.5.1-playback\MuseFlow-PC.exe`.

The 0.5.0 failure was reproduced in the visible player and independently at the CDN: Hum and Hum Nahin resolved successfully, but the returned audio address received HTTP 403, leaving playback at 0:00. The original Carefree live fixture passed, so it was insufficient coverage. The volume was 100%; audio-device initialization was not the cause of those HTTP failures.

0.5.1 verifies bounded initial/later ranges, switches to a distinct visionOS client when the automatic source is refused, and uses 64 KiB requests in libmpv's pinned curl backend. The exact selected recording is retained. Best audio remains preferred. Embedded-web and Android VR alternatives were investigated and rejected for intermittent refusals/partial-preview behavior. No new dependencies, cookies, accounts, remote decoders or global settings were introduced.

## Current verification

- Formatting, Clippy with warnings denied and optimized release build passed. **32 Rust tests passed**; the two ignored entry points are child-process fixtures used by the job teardown test.
- New HTTP fixtures cover successful ranged responses, preserved headers, forbidden/expired URLs, empty bodies, rate limiting, a prefix-only stream refusing later bytes, and HTTPS CDN host restrictions.
- The final staged executable passed **seven journeys**: Hum, Hum Nahin and Tum from isolated saved-library copies; live catalog/search playback; local playback; cancellation; online-to-local interruption.
- Each saved-song journey decoded Opus, advanced time, paused, sought to 60%, resumed and advanced beyond the seek target, then remained loaded and paused. Default Windows audio output was used at zero app volume. Separate native adapter probes identified WASAPI, stereo 48 kHz output and advancing time for all three. This verifies the audio path, not human acoustic acceptance.
- Local regression retains seek/volume, queue, Next/Previous, EOF and library behavior; cancellation/switch checks guard stale results.
- Portable candidate extraction validation: **passed; Unicode/space path, bundled runtimes, relative data and live playback**.

The previous interrupted and failed embedded-client attempts remain diagnostic history. Final outcomes are in `final-results.json`; earlier failures must not be relabeled as passes. Existing 0.5.0 tray validation is preserved in TRAY-CHECKPOINT.md, with its original pending checks.

## Boundaries and limits

The preview keeps the existing G: library; portable copies use their own adjacent data folder. Integration tests use separate G: libraries. Source/evidence remain on E:, and tools/runtime/caches/output on G:. Android is paused and unchanged. SQLite schema remains 2. No repository initialization, commits, publication or signing occurred.

Provider changes, region/account restrictions and expired streams can still prevent individual recordings. The bounded probes are not a full-file download or a guarantee covering every byte. This update does not establish cold startup, frame timing, GPU memory, long-session stability, Bluetooth/headphones/device switching, complete accessibility, actual tray-menu mouse interaction, Explorer restart, Windows 10 or clean-machine compatibility. Signed installer and the complete native/runtime license/source-fulfillment audit remain public-release prerequisites.

SimpMusic desktop uses Kotlin/Compose and libmpv with layered extraction recovery. Its useful design patterns, differences and upstream links are recorded in SIMPMUSIC-RESEARCH.md; no Kotlin source was converted into this Rust fix.

## Evidence

`E:\ALL Project & Idea Workspace\WSL BACKUP\artifacts\museflow-pc-playback-2026-11-15` contains the pre-change backup, source snapshots, redacted stream/range diagnostics, native output probes, build logs, failed-attempt history and final integration results. Temporary signed URLs were kept in memory and not written to the reports.

EXE SHA-256: `541e83326e184de98067ca349921fa6fb0bf2406bd56fe40f5ee27844db49b11`. ZIP has its own checksum.
