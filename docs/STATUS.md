# Library keyboard navigation checkpoint — 0.7.2 · 16 November 2026

Run `G:\Essential Softwares\MuseFlow-PC\Preview\0.7.2-keyboard\MuseFlow-PC.exe`.

## Completed PC milestone

The main virtualized song list supports unmodified Up/Down, Home/End and Page Up/Down. These move a transient keyboard cursor, scroll the requested row into view and leave playback selection unchanged. Enter/Space activates the focused recording’s current ID, including filtered results. Tab continues to traverse visible controls. Query/model replacement resets the cursor without changing the playing recording.

A stable parent scope holds focus while an offscreen row is created. One coalesced, deferred scroll correction fixes construction-time focus scrolling that otherwise left the last row clipped. It stops after the adjustment; no idle polling, extra worker, dependency or database setting was added. Short Home windows use a smaller hero and omit secondary decorative copy so complete song rows remain visible. SQLite remains schema 2; existing library/settings stay compatible.

## Verified on this Windows 11 PC

- Formatting, **37 Rust tests**, all-targets Clippy with warnings denied and the optimized build passed. Two ignored test entry points remain subprocess fixtures used by the resolver teardown test.
- **Four large-list native journeys passed:** dark theme at 100%, 150% and 200% renderer density and light theme at 100%. Each checks 10,000 records, Up/Down, Home/End, Page Up/Down, held-arrow input, Enter/Space, filtered stable recording IDs, empty results and minimum-size Home. Rendered top/bottom focus-border checks prevent an ID-only pass with a clipped row.
- **Three existing layout/shortcut journeys passed**, producing eight captures each at the same three densities. Physical minimum capture dimensions remain 800×600, 1200×900 and 1600×1200 pixels.
- **Six further regressions passed:** synthetic shell/filter/stable IDs; local decoding, seek, EOF, likes and playlists; paused restore; artwork/rapid selection; live online playback/search/seek; and hidden-player tray Exit. These make **13 accepted native journeys** on the staged executable.
- The extracted portable ZIP passed content hashes/CRC, Unicode-path artwork playback and tray Exit, and created its own adjacent data folder. Developer toolchain directories were removed from the child PATH for these tests; this remains validation on the existing host, not a clean-machine certification.
- Native captures were inspected for complete focused rows at the end of full/filtered lists and the compact Home layout, including light and higher-density rendering. Early clipped candidates are retained as diagnostics rather than accepted passes.

Native audio checks use silent fixtures or null output and do not certify acoustic quality. Renderer density is not actual Windows-DPI/multiple-monitor acceptance. No frame-rate, GPU-memory, cold-start or long-session memory claim was added for this patch.

## Remaining acceptance

The current PC work is complete as a preview checkpoint. Windows 10, a clean machine/installer, physical audio-device changes, headphones/Bluetooth, full screen-reader/Switch Access and actual OS-DPI/multiple-monitor journeys remain pending. The earlier accessibility focused-element report and first-capture/occlusion observations are not marked fixed. Native runtime redistribution/signing review remains incomplete, so binaries stay local and unsigned; source publication is not a stable binary release.

Android source lives in a separate workspace. Its glass/navigation experiments are not part of the PC executable.

## Evidence and files

Evidence: `E:\ALL Project & Idea Workspace\WSL BACKUP\artifacts\museflow-pc-list-keyboard-2026-11-16`. Compiler/test logs and `accepted-results.json` record this build. Earlier candidates are in the corresponding 15 November artifact folder. See [USABILITY.md](USABILITY.md) for controls and the isolated synthetic check.

EXE SHA-256: `0e1a6c4b2facbf9b9cdadfe7265517d54d4581d8b701809a578b95b3a2b9aaa4`.

The local preview retains the normal G: library; portable ZIPs use adjacent data/. Source/evidence remain on E:, tools/cache/builds/previews on G:. The preceding checkpoint is preserved in [USABILITY-CHECKPOINT.md](USABILITY-CHECKPOINT.md); earlier performance evidence remains historical in ARTWORK-CHECKPOINT.md.

Portable ZIP SHA-256: `bba32a481cc68dce2c20d3dda14372ef354d2855d6176c2d0cf862831a0e061d`. The extracted executable matches the staged binary.
