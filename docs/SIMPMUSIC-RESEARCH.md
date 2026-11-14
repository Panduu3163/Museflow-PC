# SimpMusic desktop research · 15 November 2026

## Snapshot and scope

Reviewed [SimpMusic](https://github.com/maxrave-dev/SimpMusic) at `f78a4fd6da9680e5771f4763194d2895d26914fd` and its separately maintained [core submodule](https://github.com/maxrave-dev/core/tree/8413a257cbe45d102a9f101f2760e35b549e4d9e) at `8413a257cbe45d102a9f101f2760e35b549e4d9e`. The public source was read, not built or installed. Selected upstream files and API trees are preserved as research evidence outside application source, under `artifacts/museflow-pc-playback-2026-11-15`. No SimpMusic Kotlin code was copied into MuseFlow's Rust implementation.

Yes, this repository includes a desktop application. The main project holds `desktopApp`, shared `composeApp` UI and platform entry points; much of the playback/extraction implementation lives in `core`, so examining the main repository alone misses the important parts.

## Architecture compared with MuseFlow

| Area | SimpMusic inspected implementation | MuseFlow direction |
| --- | --- | --- |
| UI/runtime | Kotlin/Compose Multiplatform, desktop JVM | Native Rust/Slint; keep the existing choice |
| Audio | libmpv behind a JNA interface and player adapter | libmpv behind a small dynamically loaded Rust adapter |
| Catalog | Shared Kotlin YouTube Music service/repositories | Rust catalog adapter informed by preserved Android references |
| Extraction | Multiple PipePipe/BravePipe attempts, local Faraday JS decoding and a remote solver tier | Isolated, on-demand pinned yt-dlp/Deno, with no remote solver service |
| Delivery | Bundled mpv natives; Gradle/Conveyor platform packaging | Explicit allowlisted Windows portable ZIP; signed installer later |
| Advanced player | Precache, crossfade, queue and retry coordination | First stabilize normal playback/seek/queue before adding multi-handle audio |

Evidence: [desktop build](https://github.com/maxrave-dev/SimpMusic/blob/f78a4fd6da9680e5771f4763194d2895d26914fd/desktopApp/build.gradle.kts), [MpvPlayer](https://github.com/maxrave-dev/core/blob/8413a257cbe45d102a9f101f2760e35b549e4d9e/media/media-jvm/src/main/java/com/simpmusic/media_jvm/mpv/MpvPlayer.kt), [MpvPlayerAdapter](https://github.com/maxrave-dev/core/blob/8413a257cbe45d102a9f101f2760e35b549e4d9e/media/media-jvm/src/main/java/com/simpmusic/media_jvm/mpv/MpvPlayerAdapter.kt).

## Most relevant lesson: extraction success is not playback success

Their JVM extractor explicitly treats a locally decoded but CDN-refused URL as a failure requiring a different attempt. The inspected sequence is local Faraday decoding through PipePipe, PipePipe with its remote decoder, then BravePipe. It does not merely repeat the same failed extraction indefinitely. See [Extractor.jvm.kt](https://github.com/maxrave-dev/core/blob/8413a257cbe45d102a9f101f2760e35b549e4d9e/service/kotlinYtmusicScraper/src/jvmMain/kotlin/com/maxrave/kotlinytmusicscraper/extractor/Extractor.jvm.kt).

MuseFlow 0.5.0's own diagnosis found the same category of problem: successful yt-dlp JSON and valid recording identity, followed by HTTP 403 from the media server. Repeating the automatic client returned another unusable address. The original Carefree smoke fixture passed while the user's Hum/Hum Nahin recordings failed. Therefore the test set must include actual failed recordings, decoder progression and later seeking.

0.5.1 adds bounded CDN verification with cancellation and a distinct visionOS client fallback for the same recording. libmpv uses bounded curl requests instead of an open-ended transfer. An embedded-web alternative was also intermittently refused, and Android VR could expose a short prefix while refusing later ranges; both were rejected as the chosen fallback. A one-byte successful probe was inadequate; the implementation checks an initial range and a later range when the advertised length supports it. These checks reduce false successes but cannot establish that every byte of every recording is playable. Public provider behavior can change.

The selected fallback stays within the existing bundled resolver. SimpMusic's remote solver approach is useful architectural context, but adopting it would introduce an additional service dependency and data flow. It was not added to MuseFlow. Browser cookies, accounts, authentication tokens and premium-only sources were not accessed.

Upstream context: [yt-dlp YouTube extractor notes](https://github.com/yt-dlp/yt-dlp/wiki/Extractors#youtube), [PO token guide](https://github.com/yt-dlp/yt-dlp/wiki/PO-Token-Guide), [mpv curl request sizing](https://mpv.io/manual/stable/#options-curl-max-request-size). The pinned yt-dlp release was already the latest stable release reported by GitHub at inspection; no dependency installation/upgrade was needed.

## Other useful patterns

1. **Bound buffers before adding precache/crossfade.** Their native player explicitly caps forward demux cache at 32 MiB and back cache at 8 MiB. Multiple player handles multiply native memory even when the language heap is bounded. These are useful experiments for MuseFlow, not measured MuseFlow defaults to copy blindly.
2. **Treat stale callbacks and track identity as first-class state.** Their adapter filters stale player errors and bounds per-recording retries. MuseFlow already uses resolver generations, stable IDs and active-media checks; regression coverage should retain those guarantees when adding features.
3. **Separate real progress from visual smoothing.** Their handler distinguishes mpv's measured time from interpolated UI position, resetting smoothing around seeks. A future smooth timeline must never fabricate playback during buffering or failure.
4. **Stop hidden visual work.** Their [DesktopFrameGovernor](https://github.com/maxrave-dev/SimpMusic/blob/f78a4fd6da9680e5771f4763194d2895d26914fd/composeApp/src/jvmMain/kotlin/com/maxrave/simpmusic/DesktopFrameGovernor.kt) suspends off-screen infinite animation and changes unfocused behavior. MuseFlow should measure its own Slint rendering rather than transplant JVM/Skiko-specific machinery.
5. **Measure native memory and the whole process tree.** Their [memory investigation](https://github.com/maxrave-dev/SimpMusic/blob/f78a4fd6da9680e5771f4763194d2895d26914fd/desktopApp/MEMORY_TUNING.md) attributes an observed problem to allocator fragmentation after experiments, not merely heap size. Its reported numbers are upstream results, not MuseFlow measurements. Keep our existing resolver-tree accounting and add long-session track-switch tests before precache.
6. **Package natives explicitly.** Their desktop build stages platform-specific mpv resources. MuseFlow's existing portable allowlist, x64/version checks and runtime hashes should remain mandatory.

## What to adopt next

Playback reliability and regression fixtures come first. Then measure bounded cache sizes, hidden CPU/GPU activity, audio-device changes and long listening sessions. Artwork caching and stronger library browsing can follow. Crossfade, remote decoders, accounts and broad visual effects require separate design and validation. This research does not justify changing Rust/Slint or resuming Android work.

## 0.6.0 application of the research

The next implementation adds per-application Windows output selection and an explicit bounded buffer policy; see [audio-output behavior](AUDIO-OUTPUT.md). It retains the exact-recording fallback fix from 0.5.1. Concrete endpoint selection is remembered, absent endpoints fall back to the system default without erasing the saved preference, and stale picker selections are validated again on the worker. Output enumeration runs on Settings refresh or track load, with no new idle polling loop.

The 32 MiB forward / 8 MiB backward demux settings and a ten-second target are starting values informed by the inspected native player. They do not establish a total RAM cap or an hours-long stability result. Native runtime readback, 120 decoded track selections, saved-output recovery and the affected online-song regressions are the relevant MuseFlow checks. Final outcomes are recorded in STATUS.md rather than inferred from SimpMusic's results.

Deferred ideas remain bounded lazy artwork caching, richer album/artist views, measured hidden-window rendering and longer listening/device-loss journeys. Multiple precached/crossfade handles, remote cipher services, account transfers and JVM-specific allocator/GC techniques are not added by this checkpoint.

## 0.7.0 current-song artwork

The next step applies the bounded/lazy artwork principle to the bottom player and lyrics panel. Local embedded front covers, folder JPEG/PNG images and exact-recording YouTube thumbnails are decoded on a dedicated worker. Request generations prevent late images from replacing the selected song's cover. The memory cache keeps 16 results, while normalized disk files have 64 MiB / 256-file bounds; startup restoration is cache-only for online artwork. See ARTWORK.md for exact limits and validation scope.

Full-list artwork and richer album/artist browsing remain later work. This change uses the existing image/Lofty packages and native Rust/Slint interfaces rather than copying SimpMusic Kotlin UI or extraction code.

## Attribution

SimpMusic is GPL-3.0 licensed; preserve upstream attribution and inspect the exact license of any file/dependency before a future source port. This document summarizes the cited implementation. The playback fix uses MuseFlow's existing APIs and official yt-dlp/mpv options; it is not a Kotlin source conversion.
