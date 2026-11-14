# Local player — 0.2.0

Normal launches now use the real local library. The sample shell is still available with `--demo`; its data is separate and is never copied into the real library.

## Use

1. Choose **Add files** or **Folder** to import local music. Folder scans include subfolders and skip symbolic links. Files stay in their original locations; MuseFlow reads metadata and stores references in its own SQLite database.
2. Use Library/Search/Liked songs to choose music. Play starts a queue from the displayed collection. The queue panel lets you choose an item without replacing that queue.
3. Use the transport, seek, volume, shuffle and repeat controls. Previous restarts the current track after three seconds, otherwise moves backward. Manual Next advances even in Repeat one; end-of-file Repeat one repeats the same recording.
4. In Library, create a playlist, choose it from the collection menu and use **+ Track** to add the current recording. Duplicate membership is ignored. Hearts persist in the local library.

Theme, volume, likes, playlists, queue order/current track and position are saved. On restart the queue is **paused** and libmpv is not initialized until playback is requested. Progress is checkpointed roughly every five seconds and on clean shutdown; abrupt termination may lose the latest few seconds. A data-folder lock prevents two copies from writing the same active library.

## Files, storage and failures

The local preview's `portable-data-dir.txt` points to `G:\Essential Softwares\MuseFlow-PC\Data`. Source remains on E:, while the compiler, Cargo caches, native runtime and compiled output stay on G:. `--data-dir` overrides the data location, followed by `MUSEFLOW_DATA_DIR`, the portable marker and the standard Windows Local AppData fallback. Settings displays the active location. Test runs always use separate artifact directories, never the personal library.

Music is deduplicated by normalized canonical path. Importing another copy at a different path creates a separate library entry; this is not acoustic fingerprinting. Reimporting a known path does not refresh changed tags yet. Supported scan extensions include MP3, FLAC, WAV, M4A, Ogg/Opus, AIFF and WavPack; actual decoder/metadata support is verified with fixtures rather than inferred from a file extension. Failed metadata reads are skipped and reported. Raw AAC/video imports are not part of this checkpoint.

Metadata is read on a scanning thread with cover-art loading disabled and bounded tag-item allocation. Scans are bounded to 100,000 discovered paths. Database transactions and audio operations run on the playback/library worker, away from the UI. Progress-only saves do not rewrite queue rows; the virtualized library and queue models are not rebuilt on each playback tick.

A missing/damaged recording or unavailable runtime shows an error and stops that request; another valid recording can be selected. Failure does not silently replace the recording with a different one. Import failures do not move, rewrite or remove source music files. Folder-level scan failures and duplicate counts are summarized after queued database writes complete.

## Current boundaries

Embedded album artwork, tray behavior, downloads, EQ, crossfade and installers remain later work. Local tracks show a neutral cover placeholder. Playlist creation/addition/viewing is implemented; renaming/deleting/reordering playlists and removing library references are not yet part of this checkpoint.

The runtime is a pinned plain-x86_64 libmpv Windows build from the source linked by mpv's installation page, with its release-asset SHA-256 checked. It embeds FFmpeg; Rust-based application code does not mean its decoder is written in Rust. libmpv is dynamically loaded by absolute path, with user configuration, scripts, its automatic YouTube resolver and video disabled. The initial volume is 35%; saved user volume is restored. Test journeys use `--null-audio` and therefore do not certify acoustic output or audio-device behavior.

Online catalog/streaming is connected in 0.3.0; see ONLINE-PLAYER.md. SQLite schema 1 migrates transactionally to schema 2, retaining local IDs/likes/playlists/queue. Older 0.2 previews reject the newer schema instead of downgrading it.
