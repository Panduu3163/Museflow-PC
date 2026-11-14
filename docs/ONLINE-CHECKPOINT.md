# Online-player checkpoint — 0.3.0 · 14 November 2026

## Delivered

Executable: `G:\Essential Softwares\MuseFlow-PC\Preview\0.3.0-online\MuseFlow-PC.exe`.

Search now offers **Your library** and **YouTube Music**. Online search is explicitly submitted with Enter/Search, supports continuation pages and cancellation, and plays the exact selected recording through an isolated on-demand yt-dlp/Deno resolver and libmpv. The existing local player remains available.

Stable recording IDs and search-queue metadata are saved in SQLite; expiring stream URLs and headers stay in memory. **+** saves one result without playing it. Restart restores the queue paused and does not start the resolver. A failed stream can refresh once for the same recording, then reports a recoverable error. Generation guards reject old search/resolve results, including when switching to a local recording.

The package includes libmpv, yt-dlp **2026.10.26** and Deno **2.9.8**, with verified download provenance and upstream notices. Rust HTTP uses pinned reqwest 0.12.28, Tokio 1.54.1 and Rustls. No separate Python/Node/FFmpeg CLI installation is required by this preview. yt-dlp's official executable contains its own Python runtime and EJS scripts; its extraction files are task-owned under G:-hosted scratch space.

Main packaged binaries total **251.03 MiB**, plus notices and library data. Keep the DLL, resolver/JavaScript folders and portable marker together. Normal data stays at `G:\Essential Softwares\MuseFlow-PC\Data`. Existing schema 1 local data migrates to schema 2; older 0.2 previews reject the upgraded database. Source/reference work stays on E:. No Android changes, Git initialization/publication, machine/user PATH changes or GPU-driver installation occurred.

## Checks passed

- Optimized release build, formatting and Clippy with warnings denied.
- **22 Rust tests passed**. Two ignored functions are child-process fixture entry points invoked by the lifecycle test, not outstanding product tests. Coverage includes original local queue/filter/persistence invariants, transaction rollback, preserving a schema-1 library/likes/queue during migration, online-ID dedup/atomicity, catalog parsing, safe stream identity/host/header validation, HTTP 503/malformed/oversized responses, prompt request cancellation, and job teardown of a parent plus descendant.
- Final packaged live journey: search -> selected Carefree / Kevin MacLeod ID -> actual Opus decode and advancing time -> pause -> seek to ten seconds -> remain paused/loaded with the decoder position settled for at least two seconds. Runtime overrides were removed; packaged dependencies were used.
- Final packaged native checks passed online queue restart without resolution, continuation appending while retaining the first page, cancellation/screen-switch stale-result rejection, interrupting an online resolution with a decoded local fixture, and a visible error for a deliberately missing resolver.
- Final local regression passed decoding/time, pause/seek/volume, Next/Previous, repeat at EOF, automatic advance, likes/playlists and queue modes. The preserved 10,000-record demo journey passed on FemtoVG and software renderers.
- Computer-use check of visible controls: focused input, typed public query, pressed Enter, saw online results, clicked selected Play, saw Opus/Pause/time advance, and closed normally. It used an isolated data folder and null output. This is not complete keyboard or screen-reader acceptance.
- SQL inspection confirmed online rows contain only `youtube:<video ID>` identities. No signed CDN addresses/headers are saved in settings. Packaged task scratch directories were empty after normal and cancelled journeys. No tracked resolver descendants remained after the measured journey.
- Secret scan found no leaks using the existing narrow reviewed public InnerTube constant exception. All **252** copied reference files and their Android originals still match their preparation hashes.

## Measured performance

Windows 11 Pro 10.0.26300, Ryzen 5 7500F / RTX 4070. Final packaged optimized binary, FemtoVG, three warm cached empty-library launches, isolated data. First-frame marker requires library-ready and AfterRendering; external polling is 10 ms. Idle measured after three seconds. Process-tree snapshots were sampled every 50 ms during the short null-output live journey.

| Measurement | Result |
| --- | --- |
| Warm first rendered frame, median | 284.29 ms (275.78–286.84 ms) |
| Idle working set, median | 89.99 MiB |
| Main application process peak working set in online journey | 153.66 MiB |
| Temporary summed process-tree peak working set during resolution | **486.07 MiB** |
| Temporary summed private bytes peak | 443.64 MiB |
| Deno process reported peak working set | 291.75 MiB |
| Tracked helper processes still running afterward | None |

Resolver-tree RAM is substantial and is included above. Individual process peaks can occur at different times; do not add them as though simultaneous. Sampling can miss shorter peaks. Warm-start and idle targets are met in this setup. The short journey does not establish the steady real-output playback target, reboot-cold startup, GPU memory, long-session behavior or frame-time smoothness. Historical measurements are preserved in LOCAL-CHECKPOINT.md and SHELL-CHECKPOINT.md.

## Remaining work and limits

Lyrics lookup, Windows media keys/metadata, tray behavior and installer/clean-machine delivery are the next product stage. Embedded artwork and advanced playlist/library editing remain pending. The search adapter currently covers songs, not a complete album/artist/recommendation experience. Online catalog results and stream availability depend on an unofficial provider endpoint, region and upstream behavior; account/verification-only content is unsupported, and browser cookies are never imported.

Full acoustic/device-loss/switch tests, Windows 10, large real-library stress, scaling/screen readers, cold startup and GPU/frame timing remain pending. Normal EOF/skip/persistence coverage is not all-recording acceptance. Automatic refresh exists, but controlled expired-URL/provider-blocking journeys are not yet fixture-certified. Empty-result copy can show "No matching tracks" while a typed query is still awaiting explicit submission; the search status text explains the action. Further UI polish remains appropriate.

The libmpv binary still imports the graphics runtime's `vulkan-1.dll`, available on this host but not bundled. Public distribution needs clean-machine validation and complete transitive native/runtime notices and corresponding-source fulfillment. This is an unsigned local preview, not that public release. The earlier occluded-window capture observation has not received a comprehensive resume/occlusion test suite; current native snapshots and manual rendered controls worked.

## Evidence

`E:\ALL Project & Idea Workspace\WSL BACKUP\artifacts\museflow-pc-online-2026-11-14` contains source backup, build/unit/native logs, final packaged snapshots, process-tree measurements, dependency inventory, redacted secret report, reference/SQL checks and provenance. Final native folders begin `final-verified-`; the measured final online journey is `verified-measured-online`. Earlier development captures are not additional final passes.

EXE SHA-256: `ef8e957d6612119a1268d56384259be5c4b8f3251820b0133c0312840facfd24`. All packaged binary hashes are recorded in SHA256SUMS.txt.
