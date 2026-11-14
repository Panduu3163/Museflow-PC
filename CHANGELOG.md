# Changelog

## 0.7.0 — Current-song artwork · 15 November 2026

- Display real selected-song artwork in the bottom player and lyrics panel.
- Prefer embedded front covers, then other embedded pictures and recognized folder JPEG/PNG images.
- Fetch the exact online recording’s thumbnail when loading/playing; restore paused online artwork from cache without a cover download.
- Keep the note placeholder for missing, corrupt or unsupported artwork without blocking audio.
- Reject stale cover completions during rapid track changes.
- Bound image input/dimensions, memory results and app-owned disk files, including interrupted writes.
- Perform I/O, HTTP, decoding, cropping and cache maintenance off the UI thread.
- Retain the 0.5.1 playback recovery and 0.6.0 saved-output behavior; verify them again with the new worker.

Full-list thumbnails, Windows media artwork and additional image formats remain later work. Existing hardware/scaling/platform checks remain pending.

## 0.6.0 — Audio-output preview · 15 November 2026

- Add Settings → Audio output, with System default and concrete Windows WASAPI endpoints.
- Remember the selected output and refresh devices from Settings or at track load.
- Use the system default when a saved endpoint is absent, retaining its preference for reconnection.
- Reject stale selections without deliberately stopping current playback; restore the previous preference after a failed save.
- Keep output enumeration and routing on the audio worker; preserve lazy audio initialization at startup.
- Bound forward/back demux buffers to 32/8 MiB with a ten-second target, retaining the 0.5.1 streaming fix.
- Make the Settings controls scrollable.
- Add output routing, persistence, missing-output and 120-track native checks; extend SimpMusic research with measured MuseFlow outcomes.

Physical hotplug/Bluetooth, complete compact/scaling/accessibility, Windows 10 and public-distribution acceptance remain pending. Buffer limits are not a total RAM cap.

## 0.5.1 — Playback recovery preview · 15 November 2026

- Fix tested music recordings stuck at 0:00 after the media server returns HTTP 403.
- Verify initial and later audio ranges before handing a stream to the decoder.
- Try the same recording through a distinct visionOS client when the automatic source is refused; preserve best audio first.
- Use bounded 64 KiB media requests in the pinned libmpv curl backend.
- Keep HTTP 403/410, other server failures and connection errors distinguishable.
- Add saved-recording playback, 60% seek/resume regression journeys and HTTP range/refusal tests.
- Record SimpMusic desktop architecture and reliability research without changing Rust/Slint or importing its Kotlin implementation.

Provider restrictions can still affect individual recordings. Existing hardware, accessibility, Windows 10 and public-distribution limitations remain.

## 0.5.0 — Tray and portable preview · 14 November 2026

- Add tray Show, Play/Pause, Previous/Next and Exit actions.
- Add saved Close window to tray, keeping normal Close as Exit by default.
- Keep playback/media controls working while hidden, restore minimized windows, and suspend hidden lyrics lookups.
- Fall back to Exit when the tray is unavailable; restore a hidden window if its tray availability is lost.
- Publish Windows playback-state changes immediately, fixing a fast Pause→Play status race.
- Embed Windows file/product version metadata from Cargo.toml.
- Add an allowlisted portable packager with relative data storage, content hashes, CRC checks and a ZIP checksum.

Signed installer, clean-machine/Windows 10 and full physical input/device acceptance remain pending.

## 0.4.0 — Desktop preview · 14 November 2026

- Add on-demand UTF-8 LRC/TXT sidecars and matched LRCLIB lyrics.
- Highlight timed lines against playback, clear stale lyrics on track changes, and allow manual reload.
- Bound lyric responses/cache, cancel old lookups, and respect service cooldowns.
- Integrate Windows Play/Pause, Previous/Next, Stop and track metadata/timeline through the existing playback worker.
- Preserve explicit Play/Pause behavior and report native-control failures in Settings.
- Restore known track duration before playback and improve the narrow lyrics panel.

Tray, physical accessory testing, artwork and clean-machine/public-release delivery remain pending.

## 0.3.0 — Online-player preview · 14 November 2026

- Search songs on YouTube Music, load more results and keep library search separate.
- Resolve the exact selected recording on demand with pinned yt-dlp/Deno; stream through libmpv.
- Cancel stale searches/resolutions and isolate resolver child processes in a Windows job.
- Save stable online recording IDs and restore queues paused without storing expiring stream URLs.
- Handle catalog failures, restricted recordings and missing runtimes visibly; refresh a failed stream once.
- Preserve existing local libraries through the schema migration. Keep helper extraction/cache files in G:-hosted scratch space.

Local preview only. Provider changes, audio-device/platform acceptance, artwork, lyrics and media keys remain limitations.

## 0.2.0 — Local-player preview · 14 November 2026

- Play local music using a lazy-loaded libmpv engine; show the actual decoded codec.
- Import music files or folders without copying or modifying the originals. Read tags off the UI thread and ignore duplicate paths.
- Save the library, likes, basic playlists, theme, volume and playback queue in SQLite.
- Use seek, volume, Previous/Next, shuffle, Repeat one/Repeat all and the queue panel.
- Restore the last queue paused, with its playback position; protect an open library against a second instance.
- Recover visibly from missing music files and an unavailable runtime. Guard rapid track selections against stale load events.
- Add sharp SVG transport icons, accessible control labels and native Windows file selection.

Embedded artwork, playlist editing/removal, online sources, lyrics, media keys, tray and installers remain pending. This preview is not a public release.

## 0.1.0 — Desktop shell · 14 November 2026

- Introduced the native Rust/Slint layout, dark/light themes, background search and virtualized sample library.
