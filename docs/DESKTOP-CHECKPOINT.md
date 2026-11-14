# Desktop checkpoint — 0.4.0 · 14 November 2026

## Delivered

Run `G:\Essential Softwares\MuseFlow-PC\Preview\0.4.0-desktop\MuseFlow-PC.exe`.

Local/online playback now includes on-demand lyrics and Windows System Media Transport Controls. Open Lyrics for a matching UTF-8 LRC/TXT sidecar or LRCLIB lookup. Timed lines highlight against playback and seeks; changing the selected recording hides stale lyrics immediately. Provider requests are cancellable, response/cache sizes are bounded, and Reload bypasses the current cached result while respecting cooldowns.

Windows Play/Pause, Previous/Next and Stop route to the existing playback worker. Explicit Play/Pause requests are idempotent. Windows receives title/artist/album/duration, playback status and timeline progress. Registration waits for an initialized library and native window; a refused second instance does not register another media session. Setup/status errors are visible in Settings. Late decoder events are discarded after Stop or while a replacement stream is being resolved.

Source stays on E:. Tools, dependency caches, compiled preview, normal library data and resolver scratch stay on G:. New dependencies are pinned Souvlaki 0.9.3, raw-window-handle 0.6.2 and windows 0.62.2; no system runtime/driver or global PATH setting was installed. SQLite remains schema 2, compatible with 0.3. No Android changes, repository initialization or publishing occurred.

The main packaged binaries total **251.30 MiB**, plus notices/data. Keep the DLL, resolver/JavaScript folders and portable marker together. Normal data remains `G:\Essential Softwares\MuseFlow-PC\Data`. This is an unsigned local preview.

## Validation

- Optimized release build and formatting succeeded; Clippy passed with warnings denied.
- **27 Rust tests passed**. Two ignored functions are subprocess fixtures launched by the tested Windows job teardown, not unfinished product tests. Existing queue/library/filter/cancellation tests remain, with additional LRC timestamp/offset/Unicode/seek and bounded-text checks, provider metadata/instrumental matching, HTTP/cache/cooldown/error/cancellation tests and media-action semantics.
- Packaged native lyrics journey passed generated UTF-8 sidecar loading, highlight after forward/backward seeks, clearing lyrics on the next recording, and return to the previous recording. The actual 820×650 light-theme overlay was rendered and inspected; removing the large placeholder made the lyrics visible in that viewport.
- A real LRCLIB request returned **58** timed lines with provider attribution. Only counts/status were retained, not the provider lyric text. Generated text is used in screenshots.
- Packaged Windows-session test inspected the matching fixture session and sent actual Windows media API Play/Pause, repeated Play/Pause, Next, Previous, Stop and restart requests. App state followed the commands; Windows reported paused/stopped states correctly. It addressed only the matching MuseFlow fixture session.
- The live streamed-recording journey also passed real Windows metadata inspection, Pause/Play, Stop and a fresh streamed restart. Null output was used; these are not acoustic or physical-key tests.
- Packaged local regression passed decoding/time advance, pause/seek/volume, Next/Previous, repeat at EOF, automatic advance, likes/playlists and queue modes. The 10,000-record demo regression passed on both FemtoVG and software renderers.
- The final packaged online journey passed live search, exact-recording resolution, decoding, progression and a settled paused seek, with development runtime overrides removed. No tracked resolver children remained, and its G:-hosted scratch directory was clean afterward.
- All **252** copied references and their Android originals still match the preparation hashes. The redacted secret scan found no leaks using the existing narrowly scoped public InnerTube constant exception.

An earlier streamed Stop→Play check failed once during development. A late-decoder-event guard was subsequently added and later checks, including the final packaged check, passed. The earlier evidence did not identify the root cause conclusively; provider/network intermittency remains an observed risk. Do not interpret these passing journeys as an all-recording or bug-free guarantee.

## Measurements on this PC

Windows 11 Pro 10.0.26300, Ryzen 5 7500F / RTX 4070, final packaged optimized executable, FemtoVG. Three warm cached empty-library launches; marker requires library-ready and first rendered frame, sampled externally at 10 ms. Idle sampled after three seconds. Short null-output online journey sampled the whole descendant process tree every 50 ms.

| Measurement | Result |
| --- | --- |
| Warm first rendered frame, median | 245.96 ms (243.17–246.37 ms) |
| Idle working set, median | 95.40 MiB |
| Main process peak working set during the online journey | 156.43 MiB |
| Temporary summed resolver-tree peak working set | 462.47 MiB |
| Temporary summed private-bytes peak | 417.85 MiB |
| Tracked helper children still running afterward | None |

Warm startup and idle working-set targets are met under these conditions. Resolver peaks include yt-dlp, Deno and its console host; individual process peaks are not simultaneous totals. Sampling can miss shorter peaks. Reboot-cold startup, GPU memory, steady long-session real-output playback and frame-time distributions are still unmeasured. Historical checkpoints are retained separately; measurements are not interchangeable across versions.

## Remaining work

Tray/close-to-tray, installers/clean-machine delivery, embedded artwork, advanced playlist editing, account libraries and full album/artist browsing remain later work. Normal close exits; minimize retains playback. Lyrics scroll manually; embedded tags, word-level karaoke, editing and translation are not implemented. The Windows backend publishes a timeline but does not wire OS timeline scrubbing; use the in-app slider.

Physical media keys/headphones/Bluetooth, audio-device loss/switching, comprehensive screen-reader/scaling checks, Windows 10 and large real-library stress remain pending. Service availability, recording restrictions, lyrics correctness and URL-expiry recovery are not exhaustively certified. The existing libmpv Vulkan-loader dependency and complete native/runtime license/source-fulfillment audit remain clean-machine/public-release prerequisites. No public release is claimed.

## Evidence

`E:\ALL Project & Idea Workspace\WSL BACKUP\artifacts\museflow-pc-desktop-2026-11-14` contains the source backup, build/test logs, final native results, generated sidecars/screenshots, fixture-scoped Windows-session commands, live provider counts, measurements, dependency inventory, secret report and reference hashes. Failed early startup/restart observations are retained as diagnostic history; successful final logs and result JSON determine the reported passes.

EXE SHA-256: `805bb06d402487387a842891f726cf303dd47b578138aacd48240087bee7ed64`. See SHA256SUMS.txt for every packaged binary.
