# Desktop audio output and buffers — 0.6.0

## Using it

Open **Settings → Audio output**. MuseFlow lists **System default** and Windows WASAPI endpoints such as speakers or an HDMI monitor. Choose an output for this application. It does not change the Windows default or another application's routing. **Refresh** rescans the current devices; opening Settings also refreshes them.

The selected endpoint is saved in the existing SQLite settings table. Restart restores it when audio is initialized. Startup still restores the queue paused without loading libmpv just to populate this list. Opening Settings initializes the native engine for enumeration, without starting music.

If a saved endpoint is absent, the picker keeps a **Saved output (unavailable)** entry and the status explains that playback uses the system default. The stored preference is retained. Refresh after reconnecting to try it again, or select System default to replace the saved preference. A new track also refreshes availability. This is not a claim of fully automatic recovery from unplugging hardware during a track.

Output changes retain play/pause intent. An entry that disappears between display and selection produces a recoverable message, resets the picker to the prior preference, and does not deliberately stop the current track. A failed settings write attempts to restore the previous native output and reports the save failure. Actual backend/device errors can still require another selection or playback retry.

The Settings controls scroll so the lower options remain reachable in smaller windows. Keyboard selection and readable descriptions use the standard Slint ComboBox. Complete screen-reader, scaling and hardware acceptance remains documented separately.

## Architecture

`audio_outputs.rs` validates and bounds the native list, filters concrete Windows WASAPI endpoints, deduplicates IDs and determines the default fallback. `engine.rs` owns enumeration, libmpv property changes and persistence on its worker. Only plain IDs/labels cross into the UI. Output models are published on enumeration/selection, not rebuilt for every progress tick. The resolver and existing stable-recording safeguards are unchanged.

The list is limited to 64 choices and 64 KiB of source JSON; descriptions are stripped of control characters and bounded. User actions send an ID, not an index assumed to remain valid. The worker verifies the ID against a fresh list before applying it. Driver IDs and endpoint choices stay local.

## Playback buffers

The pinned libmpv configuration now explicitly sets a 10-second cache target, a **32 MiB forward demux buffer** and an **8 MiB backward demux buffer**. The existing 64 KiB curl request size is preserved. These buffer limits are not a limit on total application RAM: decoder, renderer, native allocations, metadata and temporary resolver processes consume additional memory.

SimpMusic's native player uses the same forward/backward buffer sizes. This is a reference for an initial bounded policy, not evidence that every application benefits by the same amount. The project's own runtime checks and measurements determine the reported results. A low-RAM/crossfade preset is not introduced, and no Windows heap trimming or process-global allocator changes are made.

## Verification

Use isolated libraries for `--audio-output-smoke-test`, `--audio-restore-smoke-test`, `--audio-missing-smoke-test` and `--audio-stress-smoke-test`. The stress journey makes 120 selections, waiting for each requested recording to decode and advance. `--audio-runtime-probe <output.json>` reads back the configured limits from the actual bundled engine without opening the user library. See STATUS.md for final measured outcomes and limits; the existence of a check does not mean it passed.

Physical unplug/replug, headphones, Bluetooth, switching to another independently connected device, and Windows 10 remain separate acceptance checks. Muted engine tests do not establish speaker audibility. One short repeated-track journey is not an hours-long leak test.

## References

- [SimpMusic native player, pinned core snapshot](https://github.com/maxrave-dev/core/blob/8413a257cbe45d102a9f101f2760e35b549e4d9e/media/media-jvm/src/main/java/com/simpmusic/media_jvm/mpv/MpvPlayer.kt)
- [mpv audio-device option](https://mpv.io/manual/stable/#options-audio-device)
- [mpv demux buffer option](https://mpv.io/manual/stable/#options-demuxer-max-bytes)
- [Research notes](SIMPMUSIC-RESEARCH.md)

No SimpMusic Kotlin source was copied into this implementation. Its upstream attribution remains in the research notes; MuseFlow remains Rust/Slint with its existing native audio dependency.
