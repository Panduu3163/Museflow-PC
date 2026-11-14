# Implementation milestones

Preparation is complete and the first desktop shell is implemented. Current build/measurement evidence and remaining acceptance checks are in [STATUS.md](STATUS.md). Local playback, durable storage and a basic queue are now implemented. Online sources now have an initial working implementation; local hardware/platform acceptance remains partially pending. Copied Android tests remain reference material, separate from the shell's own tests.

| Stage | Deliverable | Exit check |
| --- | --- | --- |
| 0 — Preparation | Documentation, selected references/assets, licenses and verified copy manifest | Hashes match current Android files; no private data, build outputs or repository history copied |
| 1 — Shell | Rust/Slint skeleton, dark/light desktop layout, representative legal artwork and synthetic large lists | First usable frame, keyboard/scaling checks and startup/idle-memory measurements recorded |
| 2 — Local player | libmpv adapter, folder/file selection, metadata scan, SQLite, queue/seek/volume/shuffle/repeat, likes/playlists | Supported fixture formats play/seek; restart/missing-file/failure cases pass; library remains responsive during scan |
| 3 — Online | Rust catalog adapter, paginated search, yt-dlp resolver, cancellation and errors | Search -> exact recording -> resolve -> playback works; stale requests cannot win; expiry/retry and offline failure are recoverable |
| 4 — Desktop features | Basic lyrics, Windows media keys/metadata, tray with explicit Exit, Windows packaging | Controls/metadata stay consistent; keyboard/screen reader and clean-machine install/upgrade/uninstall checked |
| 5 — Acceptance | Integrated release candidate and Windows 10/11 evidence | Whole journeys, performance and memory measured on real Windows systems; outstanding limits disclosed |
| Later | Linux platform adapter, downloads, DSP, sync and opt-in glass experiments | Separate plans and platform-specific acceptance required |

## Regression scenarios

Use small licensed/generated fixtures for MP3, FLAC, WAV, AAC/M4A and Opus; verify actual decoded format rather than assuming from a filename. Exercise repeated skips, pause while buffering, seek near end, shuffle/repeat, database write failures, damaged/missing files and restart recovery. Verify URL expiry, timeout, cancellation, malformed provider data and a late result arriving after the user selects a different song. Include Unicode paths/titles, large playlists and no network.

Verify each UI action changes the correct core state and can recover after an error. Test media-device loss/switching and output errors, Windows media controls, multiple launches and shutdown with active helper processes. Test 100/150/200% scaling, narrow windows, keyboard focus and screen-reader labels. Mark any case requiring unavailable hardware or another OS as pending; do not translate untested cases into passes.

Preparation does not install Rust, Slint, libmpv, yt-dlp, a JavaScript runtime or build tools. It does not implement stages 1–5, create an installer or start a Git repository.

## 0.2.0 checkpoint

Local fixture journeys, persistence/recovery, native file selection and second-instance protection passed on this Windows 11 PC. Default WASAPI opened with a silent fixture. Full acoustic/device-switching acceptance, large real-library performance, screen readers and Windows 10 remain pending. See STATUS.md; implementation completion is not complete platform acceptance.

## 0.3.0 checkpoint

Online song search, continuation, stable recording identity, on-demand resolution, streaming, cancellation and recovery are implemented. See STATUS.md for actual native/unit results and remaining provider/platform limits. Lyrics/media controls/tray are the next product stage.

## 0.4.0 checkpoint

Basic matched/plain/timed lyrics and Windows media buttons/metadata are implemented. Native Windows-session actions are checked against both local fixtures and a live streamed recording. Stage 4 is partial: tray, installer and complete keyboard/screen-reader/clean-machine acceptance remain pending.

## 0.5.0 checkpoint

Tray lifecycle, saved close preference, hidden-player Windows media commands and portable ZIP packaging are implemented. Stage 4 still requires installer/clean-machine validation and complete accessibility/hardware acceptance. Actual notification-area mouse interaction remains pending where the computer-use shell surface is unavailable.

## 0.6.0 checkpoint

Per-app Windows output selection, saved-output fallback and explicit native buffer bounds are implemented, with 120 decoded selections and affected-song regression checks. Physical hotplug, full accessibility/scaling and clean-machine delivery remain pending. See STATUS.md.

## 0.7.0 checkpoint

Current-song cover art, lazy bounded decoding/caching and stale-result protection are implemented. Embedded/folder/online and cache-only restart checks accompany the playback/routing regressions. Full-list thumbnails, richer album/artist browsing and remaining platform acceptance are later work. See STATUS.md and ARTWORK.md.
