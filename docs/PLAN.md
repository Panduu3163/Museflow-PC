# MuseFlow PC: approved direction

## Product and platform

Build a separate Rust + Slint native desktop app for Windows 10/11 x64, followed by Linux. Keep the Android workspace unchanged and development paused. Preserve MuseFlow identity with familiar desktop navigation, excellent artwork, typography and responsiveness. This is a rewrite of platform-specific code, not an automatic Kotlin conversion.

## Architecture and interface

- Slint UI: sidebar, searchable content, virtualized album/track lists, persistent bottom player and optional queue/lyrics panel.
- Polished dark/light themes, restrained gradients, sharp foregrounds and short animations. Glass effects are deferred and disabled by default until tested.
- Rust core: catalog, library, queue, playback coordination and settings. Network, file work, decoding and database operations stay off the UI thread.
- SQLite stores desktop library/settings; artwork caches are bounded with lazy thumbnails.
- Embed libmpv behind a Rust adapter for audio decoding/streaming/seeking. The app is Rust-based; libmpv is a native external dependency.
- Port the required catalog behavior from the Android InnerTube references. Resolve playable URLs through an isolated, on-demand yt-dlp process, accounting for its supported JavaScript-runtime/EJS requirements. No resolver process remains running at idle.

## First working version

Local audio, online music search/streaming, queue, seek, volume, shuffle/repeat, likes, playlists, basic lyrics and Windows media controls. Implement the desktop shell and measure it first, then local playback/storage, then online playback, then lyrics/desktop integration/packaging. Validate complete user journeys before expansion.

Downloads, advanced DSP, synchronization and elaborate liquid-glass effects are later work. Android backup formats are reference for later interoperability; importing Android libraries/settings is not required for the first desktop milestone.

## Preparation boundary

Create only the separate workspace, documentation, selected reference copies, assets, notices and hash manifest. Do not install tools, create a Cargo application, initialize Git or publish anything during preparation. Copy actual working-tree bytes, not just the last published commit. Exclude keys, personal/device data, caches, outputs and Android build configuration.

## Acceptance

Initial targets on the user's PC: usable window within 2 seconds cold / 1 second warm; idle memory below 120 MiB; steady playback below 220 MiB. Measure the full process tree and report resolver peaks and GPU memory separately. These are targets, not results. Verify large libraries, repeated track changes, failures/cancellation, URL expiry, missing files, audio-device changes, recovery, accessibility, scaling and Windows 10/11. No bug-free/optimized claim from compilation or unit tests alone.

## Sources reviewed

- Slint Windows: https://docs.slint.dev/latest/docs/slint/guide/platforms/desktop/windows/general/
- Slint renderers: https://docs.slint.dev/latest/docs/slint/guide/backends-and-renderers/backends_and_renderers/
- libmpv: https://mpv.io/manual/master/#embedding-into-other-programs-libmpv
- yt-dlp/EJS: https://github.com/yt-dlp/yt-dlp/wiki/EJS

Dependency versions and distributable license terms must be checked and pinned when implementation starts. No dependencies have been downloaded or installed here.
