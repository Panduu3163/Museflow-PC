# Museflow PC (ON-going project)

**Work in progress · Preview 0.7.0 · Not a stable release**

A native **Rust + Slint** desktop music player for Windows, with Linux planned later. MuseFlow PC is a separate application built around MuseFlow’s music-player identity; the Android project is [Muse_Flow](https://github.com/Panduu3163/Muse_Flow).

> This project is actively being built. Features, appearance and storage formats may change, and bugs are expected. Windows 11 is the currently tested development platform; Windows 10 and clean-machine acceptance are still pending. This repository publishes source code, not a signed installer or a certified stable binary.

## What works in this preview

- Local audio import, folder scanning and playback through libmpv.
- Online catalog search and streaming using on-demand yt-dlp and Deno helpers.
- Persistent library and queue, seeking, volume, shuffle and repeat.
- Likes, basic playlists, plain/timed lyrics and current-song cover artwork.
- Windows media controls, tray behavior and saved audio-output selection.
- Background network/file/database work, virtualized lists and bounded artwork caches.

Online sources are unofficial integrations and may stop working or reject individual recordings. MuseFlow does not host audio. Missing artwork and provider failures are handled separately from local playback.

## Project status

The current milestone is the **0.7.0 artwork preview**. Formatting, Clippy, 37 Rust unit tests and the documented isolated native playback/artwork/routing journeys passed on the development PC. These results do not establish complete hardware compatibility or long-term stability.

Still pending: physical audio-device changes, Bluetooth/headphones, Windows 10, clean-machine installation, complete accessibility/scaling checks, cold-start/GPU measurements, signing and the full native-runtime redistribution audit. See [validation and limitations](docs/STATUS.md) for evidence and measurement conditions.

Downloads, advanced audio processing, synchronization and elaborate glass effects are later work. Android build 66’s reported buggy glass work is historical reference, not an approved desktop design.

## Build from source

The supported development configuration is **Windows x64, Rust 1.100.0 GNU, MinGW-w64 and Slint 1.18.1**. Dependencies are pinned in `Cargo.lock`. The Windows resource step currently requires GNU tools; MSVC and Linux builds have not been validated.

See [BUILDING.md](docs/BUILDING.md) for prerequisites, build commands, runtime configuration and isolated test instructions. The checked-in environment helper describes the maintainer’s E:/G: storage layout and is optional for other machines. Native libmpv is required for playback; online playback additionally needs yt-dlp and a supported JavaScript runtime. These binaries, personal libraries and caches are not included in this repository.

## Explore the source

| Location | Contents |
| --- | --- |
| `src/` | Rust application, catalog, storage, playback and desktop integration |
| `ui/` | Slint desktop interface |
| `assets/` | Branding, original demo/control artwork and licensed optional font |
| `tools/` | Local environment helper and allowlisted Windows packager |
| `docs/` | Architecture, milestones, research, guides and validation checkpoints |
| `reference/` | Attributed Android source/design snapshots; not desktop build inputs |

- [Plan](docs/PLAN.md), [architecture](docs/ARCHITECTURE.md), [milestones](docs/MILESTONES.md)
- [Local playback](docs/LOCAL-PLAYER.md), [online playback](docs/ONLINE-PLAYER.md), [artwork](docs/ARTWORK.md)
- [Audio output](docs/AUDIO-OUTPUT.md), [desktop features](docs/DESKTOP-FEATURES.md), [tray/packaging](docs/TRAY.md)
- [SimpMusic research](docs/SIMPMUSIC-RESEARCH.md), [performance targets](docs/PERFORMANCE.md)
- [Changelog](CHANGELOG.md), [contribution guide](CONTRIBUTING.md)

## License and attribution

MuseFlow PC is licensed under **GPL-3.0-only**. See [LICENSE](LICENSE) and [third-party notices](THIRD_PARTY_NOTICES.md). Upstream source headers and licenses are preserved. The selected Android references are an attributed snapshot, not a relicensing of upstream work.

[COPY-MANIFEST.json](COPY-MANIFEST.json) records the original preparation baseline, including copied-file hashes and the historical local workspace paths. Authored documents have evolved since preparation; their baseline hashes are not current-document checksums. Checkpoint paths refer to local test evidence, which is not included in this source repository.
