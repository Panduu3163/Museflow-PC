# Desktop architecture

## Planned module boundaries

| Module | Responsibility |
| --- | --- |
| UI | Slint views, virtualized models, navigation, keyboard/focus, theme and display state |
| Application core | Playback/queue state machine, request cancellation, action validation and state snapshots |
| Catalog | Rust HTTP/JSON catalog adapter, stable recording identity, paged results |
| Library | SQLite schema/migrations, likes/playlists/settings and local-file metadata |
| Playback | Rust wrapper around libmpv, audio-only playback, events and device errors |
| Resolver | On-demand yt-dlp process, structured results, required HTTP headers and expiry |
| Lyrics | Metadata matching, plain/synchronized parsing, provider errors and bounded cache |
| Desktop integration | Windows media controls, tray, file selection and lifecycle; Linux adapter later |

These are architectural responsibilities, not separate crates. The 0.2.0 implementation maps local behavior to `engine.rs`, `playback.rs`, `library.rs`, `queue.rs` and `local.rs`; online catalog/resolver now live in online.rs, online_ui.rs and resolver.rs; lyrics.rs/lyrics_ui.rs handle on-demand sidecar/provider lyrics, and desktop.rs integrates Windows media controls. tray.rs owns the optional tray/native-window lifecycle. Keep platform bindings isolated from the portable core. Slint communicates via typed actions and immutable state updates. The playback coordinator owns queue selection and transitions; the UI must not maintain a competing playback truth.

## Execution and data flow

UI action -> coordinator -> async catalog/resolver or library worker -> playback adapter -> typed event -> state update on the UI event loop. Tag asynchronous requests with a generation/request ID so late responses cannot replace a newer selection. Cancel stale searches/resolutions and terminate their helper processes. Serialize database writes and use transactions for multi-row mutations.

Resolve metadata IDs into short-lived stream addresses only when needed. Preserve the resolver's required headers and do not store expiring URLs as permanent recording identity. A failed/expired stream may be re-resolved once for the same current request; expose a recoverable error afterward. Do not substitute another recording silently.

Use a bounded command/event channel. Coalesce progress updates; do not republish/rebuild the full library for each playback tick. Query/page the library, virtualize rows, decode thumbnails off-thread and cap decoded-image memory.

## Native dependencies and lifecycle

libmpv is the implemented local audio engine, not a Rust decoder. yt-dlp is a helper used only on demand; its packaged Windows executable may embed Python and may require a separate supported JavaScript runtime. Rust UI/core does not mean every dependency is written in Rust.

Pin reviewed runtime versions, checksums and licenses before distribution. Invoke helper executables with argument arrays rather than shell commands; ignore unrelated user configuration, do not import browser cookies automatically, redact tokens/stream URLs in diagnostics, and clean up child processes. Keep libmpv audio-only and avoid an implicit second resolver pipeline.

Keep one durable desktop database; do not open/copy the live Android Room database as the desktop database. Restore the saved queue paused on startup. Missing local files and unsupported formats must be visible without blocking startup or corrupting queue state. Closing behavior must be explicit: normal close exits initially; any later close-to-tray preference must provide a clear Exit action.

## Portability and compatibility

First supported target is Windows 10/11 x64. Linux comes after Windows acceptance, using the same core contracts and a separate media-control/platform adapter. Audit current upstream toolchain and native-runtime support for Windows 10 before pinning dependencies. Android-only URI, permission, service, worker, notification, widget, Room and DataStore bindings must be replaced rather than copied into executable desktop code.

## Current local implementation

The worker exclusively owns SQLite, the libmpv handle and queue. A bounded 64-command channel accepts actions; progress publishes every 250 ms without rebuilding list models. Pending search requests are coalesced to one slot with generation guards. Scanning/dialog work runs on background threads; metadata batches commit transactionally and report completion only after queued writes finish.

SQLite uses WAL, foreign keys and version checks. Queue order saves are separate from small progress checkpoints. Shutdown checkpoints the current state; restart is paused. An exclusive file lock covers the library lifetime. libmpv is initialized only on first playback, user scripts/config/resolver/video are disabled, and load-event identity/path guards prevent stale rapid selections from starting the wrong requested track.

Folder scans are bounded and skip symlinks. Cover-art decoding is currently disabled, so the planned bounded thumbnail cache is not implemented yet. The actual metadata snapshot is currently held in memory with lazy visible rows; SQL pagination and real large-library benchmarks remain future optimization work.

Catalog HTTP uses an abortable Tokio request on a worker with one pending search. Provider response bodies and result pages are bounded. Resolution has one pending recording and generation-tagged completion; a killed Windows job owns its descendants. Core polls it only while resolving and rejects stale request IDs. Stable video IDs, not signed URLs or headers, go in SQLite.

The main desktop loop runs until explicit Quit. Native hide keeps the player HWND alive for SMTC; default Close and explicit Exit share normal cleanup. The tray registers only after the library is initialized and gates hiding on actual notification-area availability. Core state publication synchronizes the Windows bridge immediately to avoid stale Play/Pause state.
