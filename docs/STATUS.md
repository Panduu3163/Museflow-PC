# Current-song artwork checkpoint â€” 0.7.0 Â· 15 November 2026

Run `G:\Essential Softwares\MuseFlow-PC\Preview\0.7.0-artwork\MuseFlow-PC.exe`.

## Delivered

The selected recording's cover appears in the bottom player and lyrics panel. Local front-cover pictures take precedence over other embedded pictures, then recognized folder JPEG/PNG files. Online thumbnails use the validated recording ID on a fixed YouTube image host. Restoring paused online artwork checks the cache without a fresh cover request. Missing/corrupt art keeps the existing note icon and does not stop audio. Full-list thumbnails and Windows media-control artwork remain later work.

File/tag reads, HTTP, image decoding/cropping, PNG encoding and cache maintenance stay on a single worker with one replaceable pending request. Generation/selected-ID checks keep late results out of the active UI. Finished pixel data is imported into Slint on the event loop. No new idle timer or helper process was added. The existing image 0.25.10 package is now a pinned direct dependency; no packages/toolchains/runtimes were downloaded or upgraded. SQLite remains schema 2.

Inputs are bounded to 4 MiB, 4096 pixels per side and 4,194,304 total pixels before decoding. Decoder allocation limits are requested as an additional best-effort safeguard. Completed covers are 256Ã—256 RGBA; the memory cache holds 16 results, at most approximately 4 MiB retained pixels plus metadata. Recognized app-owned disk entries, including interrupted-write files, have a 64 MiB / 256-file write-time quota. Unrelated files/directories are not evicted. These are cache limits, not a whole-process RAM guarantee. See ARTWORK.md.

## Verified

- Formatting, Clippy with warnings denied and optimized build passed. **37 Rust tests passed**; two ignored entry points are child-process fixtures used by the tested resolver teardown.
- Decoder/cache tests passed malformed/oversize images, excessive dimensions, JPEG/PNG decoding, local-source invalidation, persistent folder art, bounded negative results, scoped quota/count eviction and interrupted-write accounting. HTTP fixtures passed errors, declared oversize rejection and in-flight cancellation.
- **Four final packaged artwork journeys passed:** embedded front-cover priority over back/folder art, folder fallback, corrupt/missing art and thirty rapid selections; paused local disk-cache restore; exact online cover alongside playback; paused online cache restore without initializing audio or resolving a stream. Generated fixture paths include Bengali characters. Unit colour checks and native pixel checks verify the cover matches the selected fixture.
- **Four audio regressions passed:** concrete/default route changes with pause preservation, stale selection rejection while playing and Next; saved routing restore; absent saved output falling back without losing its preference; and 120 decoded selections with correct identity/time progression.
- **Seven playback regressions passed:** Hum, Hum Nahin and Tum with Opus and 60% seek/resume/stable pause, live search playback, local controls/EOF/library behavior, cancellation and online-to-local interruption.
- Native screenshots were inspected for the bottom-player image, readable compact artwork in the lyrics panel, correct source image and fallback behavior. This does not replace physical hearing, complete keyboard/screen-reader or scaling acceptance.
- All 252 copied reference files and their Android originals still match their preparation hashes. Redacted source secret scanning passed with the existing narrow public InnerTube constant exception.
- Portable candidate: **Unicode/space-path extraction, content hashes, bundled runtime, relative data and live playback passed**.

## Measurements on this PC

Windows 11 Pro 10.0.26300, Ryzen 5 7500F / RTX 4070, optimized packaged build. Three warm cached empty-library runs; usable frame requires library ready plus AfterRendering, external polling at 10 ms and idle sampled after three seconds. Online process-tree sampling is 50 ms; local 120-selection sampling is 100 ms. Generated/default Windows tests are muted; the short measured online journey uses null output. These results are not acoustic acceptance.

| Measurement | Observed |
| --- | --- |
| Median warm first frame | 255.94 ms |
| Median idle working set | 96.38 MiB |
| 120-selection local journey | 90.11 s |
| Local journey main-process peak working set | 132.55 MiB |
| Local journey final sampled working set | 127.90 MiB |
| Short online sampled total process-tree peak | 498.28 MiB |
| Tracked resolver helpers remaining afterward | 0 |

Resolver RAM remains substantial and is included. Separate journeys and individual process peaks must not be added as simultaneous totals. Sampling can miss shorter peaks. Cold startup, GPU memory, frame timing, many hours of listening and diverse real-library cover memory remain unmeasured; do not infer leak freedom from this short stress run.

## Remaining limits and workspace

Unplug/replug, Bluetooth/headphones, routing to a second independent physical device, complete compact/scaling/accessibility, actual tray-menu mouse/Explorer restart, Windows 10, clean-machine delivery, signing and the complete native/runtime license/source-fulfillment audit remain pending. The earlier first-capture/occlusion observation is not marked fixed. Online provider restrictions can still prevent individual recordings or thumbnails; these failures are handled independently. Artwork edits during the same active selection are not watched automatically.

The normal preview retains the existing G: library. Each portable copy uses adjacent data/, and archive inputs exclude all user data/cache files. Tests use separate G: folders. Source/evidence remain on E:, dependencies/build/cache/output on G:. Android development remains paused. This source snapshot is being published as an ongoing preview, with binaries and local test data excluded. The verification described above predates repository publication; prior checkpoints preserve their original scope.

## Evidence

`E:\ALL Project & Idea Workspace\WSL BACKUP\artifacts\museflow-pc-artwork-2026-11-15` contains the pre-change backup, tests/build logs, generated cover fixtures/checks, final native/playback/audio results, inspected snapshots, memory samples and archive validation. Final result JSONs determine the reported passes. Debug and earlier candidate observations are diagnostic history.

EXE SHA-256: `d9dfae294e8e8f1e60e848925cc72eaa282ea0a9fd1e2cc83c7a25af2579d446`. The ZIP has its own checksum.
