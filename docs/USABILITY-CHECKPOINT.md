# Compact layout and keyboard checkpoint — 0.7.1 · 15 November 2026

Run `G:\Essential Softwares\MuseFlow-PC\Preview\0.7.1-usability\MuseFlow-PC.exe`.

## Delivered

Settings now uses the available scrollable height; at the minimum 800 × 600 logical size, the audio-output selector is fully visible rather than being cut off. Its status/help content remains in that scroll area. Volume has a visible numeric label. Startup uses the declared preferred window size so the backend can apply its scale factor.

Ctrl+1–5 navigates sections and Escape dismisses a side panel after its child controls decline the event. Track rows accept Tab focus and Enter/Space activation, with visible outlines. Play/View names include the song and artist; playlist fields have explicit names, Shuffle reports its state, and long button text elides without shortening its accessible name. Held activation-key repeats are ignored. Like transfers focus to a stable scope before the row model refreshes. See [USABILITY.md](USABILITY.md).

No dependency upgrades, runtime installations or SQLite schema changes were required. The added native journey is opt-in and adds no running idle timer during ordinary use. Test data and binaries remain on G:, with source/evidence on E:; Android remains paused.

## Verified on this Windows 11 PC

- Formatting, **37 Rust tests** and Clippy with all-targets/warnings denied passed; two ignored entry points remain child-process fixtures. The optimized build passed.
- **Three native layout/keyboard journeys passed**, each with eight captures, at 100%, 150% and 200% renderer density. Minimum captures are respectively 800×600, 1200×900 and 1600×1200 physical pixels. Assertions check the requested scale-aware dimensions. Dark/light Settings, Home, Library, Queue and Lyrics layouts were sampled visually.
- Native key events verify Ctrl+1–5, rejection of Ctrl+Shift+number as a section shortcut, Escape, Return activation and ignored held-key repeats. The test rejects absent, missing, blank and flag-like data-directory arguments before opening a library; all four guard cases passed.
- **Six further native regressions passed:** synthetic 10,000-track shell/filter/stable-ID journey, local playback/seek/EOF/likes/playlists, paused restart, artwork/rapid selections, live online search/stream/seek and hidden-player tray Exit.
- Manual native-window interaction confirmed Enter-to-play and Space-to-like in the candidate, then confirmed the final build keeps Ctrl+5 working after Like replaces the row model. The named controls are present in the accessibility tree. This does not certify screen-reader focus announcements.
- The portable ZIP passed content hashes/CRC, Unicode-path extraction, bundled-runtime artwork playback, hidden-player tray Exit and adjacent-data creation checks. No library/cache data was present in the archive.
- All **252 copied references** still match their preparation hashes. Redacted source secret scanning passed with the existing exact public InnerTube constant exception.

All native audio regressions use null output or silent generated fixtures; they do not establish acoustic quality. Early cropped density captures and a tray attempt missing its required imported fixtures are diagnostic history, not accepted passes. The corrected final results and dimension assertions determine the findings above.

## Remaining acceptance

Physical unplug/replug, Bluetooth/headphones, a second independent output device, full screen-reader/keyboard-list coverage, actual Windows DPI and multi-monitor transitions, Windows 10, clean-machine delivery, actual notification-area mouse/Explorer restart and the complete native-runtime redistribution audit remain pending. The accessibility automation focused-element report did not reliably follow custom-control focus; narrated focus needs separate validation. The earlier first-capture/occlusion observation is not marked fixed.

Renderer-density overrides are not OS-DPI certification. Long-session memory, GPU cost and cold startup were not measured again for this patch. Earlier performance numbers and artwork acceptance remain historical in [ARTWORK-CHECKPOINT.md](ARTWORK-CHECKPOINT.md). No stable/bug-free release claim is made.

## Evidence and preview

Evidence: `E:\ALL Project & Idea Workspace\WSL BACKUP\artifacts\museflow-pc-usability-2026-11-15`. The `*-guard.log` compiler/test logs, `accepted-*` native captures, `accepted-results.json` and `final-manual-focus.json` describe this candidate. Preliminary files preserve diagnostic attempts.

EXE SHA-256: `3398479110aa8d52971fee7f9860cd0afd5e5b57d1b7b0ef4bf7dcea4759e8c6`.

The local preview retains the existing G: library; portable copies use adjacent `data/`. Runtime binaries, libraries, caches and local evidence are excluded from the source repository. The portable archive remains an unsigned local preview, not a certified installer or public binary release.

Portable ZIP SHA-256: `1cd8ae95d9d4cbfd11da27c3c8428fa84f850211636e8ed7bdd35c2f7818908f`. Its native executable matches the tested staging binary.
