# Current-song artwork — 0.7.0

The bottom player and lyrics panel display artwork for the selected recording. The existing note icon remains when no usable image is found. The selected image is loaded lazily; importing a library does not extract or download every cover.

## Sources

For a local file, MuseFlow first tries embedded front-cover pictures, then other embedded pictures. If none decode, it tries `cover.jpg`, `cover.png`, `folder.jpg` and `folder.png` in the audio file's directory. JPEG and PNG are supported. A damaged embedded picture can fall through to another image or a folder cover; a bad cover does not block audio playback.

For an online recording, the worker constructs the standard YouTube thumbnail address on `i.ytimg.com` from the validated recording ID. It does not use search-by-title or substitute artwork from a different search result. A video thumbnail can be a music-video image rather than an album sleeve. The centre is cropped to a 256×256 image for the current controls.

Fresh online artwork is requested while the user loads/plays that recording. Restoring a paused online queue at startup checks the local artwork cache without downloading a missing cover. No browser cookies, account data, resolver output or arbitrary remote addresses are used for cover requests. Redirects are not followed. Network failures leave the placeholder and do not replace playback errors or alter the queue.

## Work and cache limits

- One worker and one pending request; a newer selection replaces pending work.
- Network requests have a three-second connect timeout and six-second total timeout, with cooperative cancellation checked every 40 ms. Local reads/decoding complete on the worker; filesystem latency is not a hard wall-clock bound.
- Encoded images are limited to 4 MiB. Headers are checked before decoding: dimensions at most 4096 per side and at most 4,194,304 pixels. Decoder allocation limits are also requested, but those are best-effort limits in the image library, not a process RAM ceiling.
- Completed images are 256×256 RGBA. The memory cache holds at most 16 results, approximately 4 MiB of retained pixel data plus metadata. Missing results share that entry limit and have a 60-second cooldown; successful in-memory entries expire after ten minutes.
- The disk cache is `<data folder>/artwork`, using normalized PNGs. App-owned entries are bounded to 64 MiB and 256 files on writes. Oldest written entries are evicted first; only recognized flat cache filenames are removed. Unrelated files/directories are not pruned. If enough space cannot be reclaimed, the new disk write is skipped while the image can still display.
- Disk entries are reused for up to seven days. Local keys include the audio file and recognized sidecars' sizes/modification times. Switching away and back, or restarting, re-evaluates these keys. There is no filesystem watcher for artwork edits during the same selection.

File/tag reads, HTTP, decoding, cropping, compression and cache maintenance all run off the UI thread. The event loop receives only finished pixel data and creates the Slint image there. It checks both the request generation and selected track before displaying a result, so a late cover cannot replace the current song's image. No new idle polling timer or helper process is added.

The image decoder is the already-present `image` 0.25.10 package, now explicitly pinned as a direct dependency with PNG/JPEG enabled. No toolchain, native runtime or resolver upgrade was needed. SQLite schema remains 2. Portable packages exclude the data/cache folder.

## Verification and scope

Generated WAV/tag/image fixtures verify embedded-front priority over back/folder artwork, folder fallback, damaged/missing images, thirty rapid selections and disk-cache restoration. HTTP fixtures exercise errors, declared oversize responses and cancellation. Decoder/cache checks cover malformed input, excessive dimensions, bounded negative entries, local modification invalidation and scoped eviction. Native online checks verify the exact selected record's cover and cache-only paused restoration. Final results and measurements are recorded in STATUS.md.

This checkpoint covers the current song. Lazy thumbnails for the full track list, richer albums/artists views, artwork in Windows media controls, embedded WebP/other formats, cache-management settings and a metadata editor remain separate work. Existing hardware/accessibility/platform acceptance limits are retained.

## References

- [Slint image/pixel-buffer API](https://docs.slint.dev/latest/docs/rust/slint/struct.Image)
- [Image decoder limits and their guarantees](https://docs.rs/image/0.25.10/image/struct.Limits.html)
- [Lofty source, pinned version](https://docs.rs/crate/lofty/0.25.5/source/)
- [SimpMusic research and remaining priorities](SIMPMUSIC-RESEARCH.md)
