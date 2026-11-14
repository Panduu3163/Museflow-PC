# Lyrics and Windows media controls — 0.4.0

## Lyrics

Choose a song and open **Lyrics**. MuseFlow first looks for a UTF-8 `.lrc` or `.txt` file with the same base name beside a local audio file. It reads the sidecar without modifying either file. Timed LRC lines highlight against playback position, including forward/backward seeks and track changes. Plain lyrics remain readable without a moving highlight. Scrolling is currently manual; word-level karaoke timing is not implemented.

When no sidecar is available, opening the panel requests a matching record from LRCLIB using the title, artist, album when known, and duration. No lookup runs for a closed panel. Unknown artist/title metadata produces guidance instead of a broad search. The response must match normalized title/artist and known duration within three seconds; the app does not display a different fuzzy search result. Instrumental and missing-record states are explicit.

**Reload lyrics** rereads a sidecar or invalidates the current in-memory provider result, while retaining service cooldown rules. Closing the panel or changing recordings invalidates the previous request. Old lyrics are hidden immediately when the selected recording changes.

Limits: 128 KiB text, 2,000 lines and 24 cached provider documents per session. Provider bodies are capped at 512 KiB. HTTP errors, malformed responses, cancellation, mismatched metadata and service cooldowns produce visible status messages. HTTP 429/503 `Retry-After` seconds are honored. Lookups run sequentially on a worker, with a short interval between requests; caches and network work stay outside the UI thread. Provider text is treated only as display data.

Only metadata is sent to LRCLIB; audio, local paths, browser cookies and account credentials are not uploaded. No API key or registration was created. Lyrics availability and correctness depend on the sidecar/provider. A successful fixture does not guarantee every recording has lyrics. Embedded lyric tags, editing, publishing and translation are later work.

## Windows controls

The app registers Windows System Media Transport Controls after its native window is available. Play, Pause, Previous, Next and Stop route to the existing playback/queue worker. Explicit Play/Pause commands are idempotent. Stop cancels a pending resolution, stops audio and returns position to zero. Selecting Play again reloads the same queue entry.

Windows receives title, artist, album, duration, playback status and timeline progress. The bridge publishes playback-state changes immediately after core updates and bounds progress updates. Unchanged metadata is not republished. A restored queue stays paused in MuseFlow; Windows reports no active audio until the engine is loaded. Registration/metadata failures are visible in Settings and leave in-app controls available.

Souvlaki 0.9.3 provides the Windows binding. Its Windows backend currently publishes a timeline but does not wire timeline scrubbing requests; use the in-app seek slider. Device/media-key routing depends on Windows' active-session policy. The automated test addresses only the matching MuseFlow fixture session and does not send commands to unrelated players. Physical keyboard/headphone/Bluetooth controls still need hardware testing.

Normal close exits by default. Version 0.5 adds opt-in close-to-tray and explicit Show/Exit controls; see TRAY.md. Autostart, signed installer and Start-menu integration remain later work.

## Storage and compatibility

Source remains on E:. Tools, dependencies, build output, preview binaries, normal library data and resolver scratch space remain on G:. The new Rust dependencies are in the existing G:-hosted Cargo cache. No additional system runtime, driver, registry setting or global PATH entry was installed for this milestone. SQLite remains at schema 2, compatible with the 0.3 preview.

The previous native/runtime distribution and Windows 10 acceptance limits still apply. See STATUS.md for actual measurements and test outcomes.

## Sources

- [LRCLIB API documentation](https://lrclib.net/docs)
- [Souvlaki 0.9.3 documentation](https://docs.rs/souvlaki/0.9.3/souvlaki/)
- [Slint native-window handle lifecycle](https://docs.slint.dev/latest/docs/rust/slint/struct.Window#method.window_handle)

The API references describe integration contracts; the validation report records which flows were actually tested.
