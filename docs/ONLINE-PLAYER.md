# Online music — 0.3.0 preview

In **Search**, choose **YouTube Music**, enter a song or artist, and press Enter or Search. The other search source, **Your library**, filters saved/local tracks without contacting the catalog. Typing alone does not send an online request. Cancel or switch screens to discard an in-flight search. Load more requests the next catalog page; results are bounded to 500 songs per search.

Play resolves the selected recording's stable YouTube ID through the pinned yt-dlp runtime, then streams audio through libmpv. It never substitutes a different search match. Choosing another recording invalidates the previous resolution; closing the app cancels it. A Windows job terminates the resolver's descendants when its task ends. No resolver or JavaScript process runs while idle.

Search results are metadata, not downloaded songs. Playing a search collection saves its queue metadata to the library so restart can restore the same recording IDs. **+** saves a result to the library without starting playback. Likes and playlist actions are available on saved tracks in Your library. Online queue entries resolve again when played after restart; temporary signed stream URLs and request headers are never stored in SQLite.

An unavailable recording, network problem, provider verification request or missing runtime produces a visible error. A stream that fails while opening/playing is resolved once more for the same recording; playback then stops with an error if the refresh also fails. A later manual Play can retry. Restricted/account-only content is not supported. The app does not access browser cookies or sign in to an account.

## Runtime and storage

- yt-dlp **2026.10.26**: `G:\Essential Softwares\MuseFlow-PC\yt-dlp\2026.10.26`.
- Deno **2.9.8**: `G:\Essential Softwares\MuseFlow-PC\JS-Runtime\v2.9.8`.
- Rust HTTP catalog client uses pinned reqwest 0.12.28 with Rustls; no private API key.
- Source and test evidence stay on E:. Dependencies, compilation and packaged previews remain on G:.

The preview looks for `resolver/yt-dlp.exe` and `javascript/deno.exe` beside the executable. Developer overrides are `MUSEFLOW_YTDLP` and `MUSEFLOW_DENO`. The build helper sets this PC's G: paths only for its process.

Official release-asset SHA-256 digests are checked before using downloads. The resolver ignores unrelated config/plugins, disables its persistent cache and auto-update, permits no remote component downloads, and receives only an app-generated watch URL for a validated ID. EJS scripts are included in the official yt-dlp executable; Deno is passed explicitly. Media is streamed, not written as a download.

## Limits

This is a local preview using an unofficial music catalog endpoint. Provider changes, regional availability, verification requirements and upstream extraction regressions can break online playback. Public distribution still requires the native/runtime transitive license audit and clean-machine/Windows 10 checks documented in STATUS.md. Do not treat a successful live fixture as a guarantee for every recording.

Artwork, account libraries, albums/artists browsing, recommendations and downloads remain later work. Lyrics and Windows media buttons/metadata are connected in 0.4; see DESKTOP-FEATURES.md. No Android changes or repository publishing are part of this milestone.

## Sources

Catalog request and item semantics were adapted from the preserved attributed Android InnerTube/YouTube references, including Echo-Music adaptations; their upstream notices remain intact.

- [yt-dlp EJS runtime setup](https://github.com/yt-dlp/yt-dlp/wiki/EJS)
- [yt-dlp CLI options and output](https://github.com/yt-dlp/yt-dlp#usage-and-options)
- [yt-dlp pinned release](https://github.com/yt-dlp/yt-dlp/releases/tag/2026.10.26)
- [Deno pinned release](https://github.com/denoland/deno/releases/tag/v2.9.8)

Actual build/native outcomes are recorded separately in STATUS.md; implementation descriptions are not test passes.

Resolver TEMP/TMP and DENO_DIR are overridden only for each child invocation, inside its G:-hosted owned scratch directory. That directory is removed after the job exits. The 0.3 SQLite upgrade preserves local records; use the newer app for the upgraded library.

## 0.5.1 recovery

Successful resolver JSON is not proof of usable audio. The worker performs cancellable, bounded initial/later range checks against the permitted HTTPS media CDN before opening playback. HTTP 403/410 from the automatic source triggers one visionOS-client resolution for the exact same video ID. Other server errors and connection failures remain distinct. No signed addresses or headers are persisted.

The pinned libmpv curl backend uses 64 KiB requests; an open-ended read was rejected for affected recordings. Best audio remains the first choice. The three failed-song fixtures decoded Opus in validation; fallback format availability remains provider-controlled. HLS/live streams are not added as a feature by this patch.

Embedded-web extraction was intermittently refused; Android VR could expose an initial preview while refusing later bytes. Neither is the selected recovery source. Tests now include the failed saved recordings, a paused seek to 60%, resumed time progression and stable pause. Partial range verification cannot guarantee an entire recording; do not describe a single test fixture as exhaustive provider coverage.
