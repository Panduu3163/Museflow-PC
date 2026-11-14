# MuseFlow PC notices

The project GPL-3.0 license text is preserved in [LICENSE](LICENSE). The selected Android source is an attributed reference snapshot, not code relicensed by this preparation.

The InnerTube reference identifies adaptations from [Echo-Music](https://github.com/EchoMusicApp/Echo-Music), under GPL-3.0. Preserve existing source headers and upstream attribution when porting/adapting implementation.

Original Android notices are preserved unchanged in [reference/android/THIRD_PARTY_NOTICES.md](reference/android/THIRD_PARTY_NOTICES.md). Corresponding bundled license texts are under `reference/android/app/src/main/assets/licenses/`. Those notices describe Android components; they do not mean Compose, Haze, Media3, NewPipeExtractor or the JVM are installed desktop dependencies.

Doto font is copied with its SIL Open Font License in [assets/fonts/Doto-OFL.txt](assets/fonts/Doto-OFL.txt). It is optional typography. Existing MuseFlow branding and historical screenshots remain project reference assets; no Spotify branding or assets were copied.

The shell now uses Slint/slint-build 1.18.1. Slint's crate declares GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0; this GPL-3.0 project uses the GPL option. Rust dependencies are pinned in Cargo.lock. Original demo SVGs are covered by the project license. The GNU compiler and Rust toolchain are development tools on G:, not application runtime requirements.

The local preview uses libloading 0.8.9 (ISC), Lofty 0.25.5 (MIT OR Apache-2.0), rfd 0.18.0 (MIT), rusqlite 0.42.0 (MIT) and bundled SQLite 3.54.1 (public domain). Full Cargo package/version/license metadata is recorded in the milestone dependency inventory. License metadata is an inventory, not a completed legal distribution review.

The native audio runtime is mpv v0.42.0-38-gbcde6b93c, FFmpeg N-128766-gcd4e96eb3, client API 2.6, from the shinchiro Windows build dated 20261111 (plain x86_64). Download provenance and SHA-256 are included beside the preview. mpv's upstream Copyright, GPL and LGPL texts are preserved under the runtime licenses folder. This binary statically includes additional libraries/codecs; those transitive notices and corresponding-source obligations still require a complete audit before public distribution. No claim of a complete redistribution package is made for this local preview.

The selected runtime imports the graphics runtime's vulkan-1.dll; a GPU-driver installation provides it on this host. Windows system/driver DLLs are not copied into the preview. The online preview now bundles yt-dlp 2026.10.26 and Deno 2.9.8 from verified official release assets. yt-dlp source is Unlicense, while its Windows executable includes third-party components with additional terms; upstream LICENSE and THIRD_PARTY_LICENSES.txt are preserved. Deno uses MIT with additional third-party/V8 terms requiring complete release review. reqwest 0.12.28 (MIT OR Apache-2.0), Tokio 1.54.1 (MIT), serde_json 1.0.154 (MIT OR Apache-2.0) and Rustls transitives are pinned in Cargo.lock. A full shipping license/source-fulfillment audit remains pending. This source repository does not distribute those native runtime executables; a binary-release audit remains pending.

The 0.4 desktop preview uses Souvlaki 0.9.3 (MIT), raw-window-handle 0.6.2 (MIT OR Apache-2.0 OR Zlib), and windows 0.62.2 (MIT OR Apache-2.0) with Windows system media APIs. Souvlakiâ€™s license text is preserved in the preview. LRCLIB is an on-demand external lyrics service; requests identify MuseFlow and do not create an API key/account. Lyrics retain their applicable rights and are not bundled as project-owned assets. Live-test evidence stores counts/attribution rather than provider lyric text.

The 0.5 Windows tray uses tray-icon 0.26.1 and Muda 0.21.2 (MIT OR Apache-2.0). Their license files are preserved in the preview. The tray logo derives from existing MuseFlow branding and uses a code-rendered dark brand background for theme contrast. The complete transitive native/runtime redistribution audit remains pending; the portable archive is a local preview.

Version 0.7 promotes the already-present image 0.25.10 package (MIT OR Apache-2.0) to an explicitly pinned direct dependency for bounded PNG/JPEG cover decoding and normalization. Existing Lofty 0.25.5 reads embedded cover metadata. No SimpMusic implementation or third-party album image is included as a packaged asset; fetched user artwork stays in the excluded data cache. See dependency-inventory.json and ARTWORK.md.
