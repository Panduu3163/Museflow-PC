# Audio-output checkpoint — 0.6.0 · 15 November 2026

Run `G:\Essential Softwares\MuseFlow-PC\Preview\0.6.0-audio\MuseFlow-PC.exe`.

## Changes

Settings now lists System default and Windows WASAPI endpoints, saves the selected output and rescans on Settings/Refresh or track load. A missing saved endpoint falls back to the default without erasing its preference. A stale selection is rejected and a failed settings save attempts to restore the prior native output. Enumeration, routing and persistence run on the audio worker; empty-library startup does not initialize audio just to list devices. Output models are updated on these actions rather than every progress tick.

The native demux buffers are explicitly bounded to 32 MiB forward / 8 MiB back, with a ten-second target. The existing 64 KiB curl requests and visionOS same-recording fallback remain. These values are informed by SimpMusic's native player; no Kotlin implementation was copied. No dependencies were added or upgraded. See AUDIO-OUTPUT.md and SIMPMUSIC-RESEARCH.md.

## Verified on this PC

- Formatting, Clippy with warnings denied and optimized build passed. **33 Rust tests passed**; two ignored entry points are process fixtures used by the tested job teardown.
- **Four native audio journeys passed:** explicit Speakers/default switching with pause/position preservation, stale-selection rejection while playing, resume/Next; saved endpoint/paused queue restore; unavailable saved endpoint using default without losing its preference; and 120 selections with correct identity and decoded time progression.
- Runtime readback from the final executable confirmed all four configured cache/request values in the actual bundled libmpv.
- **Seven playback regressions passed:** Hum, Hum Nahin and Tum with Opus/default Windows output at zero volume, 60% seek/resume/stable pause; live catalog/search playback; local controls/EOF/library behavior; cancellation; online-to-local interruption.
- Manual native picker check passed the explicit save-rejection fixture: a SQLite trigger refused only audio_output writes, the message appeared, the picker restored Speakers and read-only SQL confirmed the old preference. Local decoding worked afterward. The Settings layout was readable at 1240×820. This is not full accessibility/scaling acceptance.
- Portable candidate: **Unicode/space-path extraction, hashes, bundled runtime, relative data and playback passed**.

## Measurements

Windows 11 Pro 10.0.26300, Ryzen 5 7500F / RTX 4070, optimized packaged executable. Warm launches use three cached empty-library runs; first usable rendering requires library ready plus AfterRendering, external polling at 10 ms, idle sampled three seconds later. Online tree sampling is 50 ms; local stress sampling is 100 ms. Audio checks are muted, not human listening tests.

| Measurement | Observed |
| --- | --- |
| Median warm first frame | 278.17 ms |
| Median idle working set | 94.36 MiB |
| 120-selection local journey | 92.50 seconds |
| Local journey main-process peak working set | 127.84 MiB |
| Local journey final sampled working set | 113.34 MiB |
| Short online journey sampled total process-tree peak | 454.81 MiB |
| Tracked resolver children remaining after online journey | 0 |

Transient resolver-tree RAM remains substantial and is included above. These are separate journeys, not simultaneous totals or an A/B proof of memory savings. Sampling may miss shorter peaks. A 93-second repeated-selection test does not establish hours-long leak freedom, cold startup, GPU memory or frame timing. Configured buffer ceilings are not an application RAM cap.

## Limits and workspace boundaries

Actual unplug/replug, headphones/Bluetooth and routing to a second independent physical device remain pending. The tested explicit Speakers endpoint may be the same hardware as System default. Refresh/next-track fallback is implemented; automatic mid-track reconnect behavior is not claimed. Compact resizing attempts did not establish a changed window size, so compact/scaling checks remain pending. The earlier blank first capture after activation was seen again; the next capture/input rendered normally, and that capture/occlusion observation is not marked fixed.

Complete keyboard/screen-reader, tray-menu mouse/Explorer restart, Windows 10, clean-machine, signed installer, and native/runtime license/source-fulfillment acceptance remain pending. Provider restrictions can still prevent individual online recordings. Prior 0.5.1 playback and 0.5.0 tray checkpoints retain their original scope and caveats.

Normal preview data stays in the existing G: library; portable copies use adjacent data/. Tests use separate G: folders. Source/evidence stay on E:, tools/cache/build/distribution on G:. SQLite remains schema 2. Android and all copied references remain unchanged. No Git initialization, commits, publication, signing, machine settings or global PATH changes were made.

## Evidence

`E:\ALL Project & Idea Workspace\WSL BACKUP\artifacts\museflow-pc-audio-2026-11-15` contains the pre-change backup, build logs, native/runtime/SQL checks, snapshots, memory samples, regression logs and archive checks. Earlier check-development failures are not product passes; the final result JSONs determine the reported outcome.

EXE SHA-256: `b4c0c52a4e12009af90df9aebe91e55ae0c8193c99abf61e853fd5c43150464a`. The ZIP has a separate checksum.
