# Building the local player

Current toolchain: Rust 1.100.0, `x86_64-pc-windows-gnu`, Slint/slint-build 1.18.1 and portable WinLibs GCC 16.2.0 / MinGW-w64 UCRT 14.0.0. Exact dependencies are pinned in Cargo.lock. SQLite 3.54.1 is compiled from rusqlite's bundled source.

## Prerequisites on another Windows PC

Install Rust 1.100.0 with the `x86_64-pc-windows-gnu` target, rustfmt/Clippy, and a compatible x64 MinGW-w64 toolchain providing `x86_64-w64-mingw32-gcc` and `windres` on PATH. The currently tested compiler is listed above; other configurations are unverified. Keep compiler paths free of spaces, or use a temporary drive alias as documented below. No Android/JVM/Node.js runtime is required.

From your clone, using your configured GNU environment:

```powershell
rustup target add x86_64-pc-windows-gnu --toolchain 1.100.0
cargo fmt --all -- --check
cargo test --locked --target x86_64-pc-windows-gnu
cargo clippy --locked --target x86_64-pc-windows-gnu --all-targets -- -D warnings
cargo build --locked --release --target x86_64-pc-windows-gnu
```

The first build downloads the locked Rust dependencies. Runtime configuration is separate: set `MUSEFLOW_MPV_LIBRARY` to your libmpv DLL, `MUSEFLOW_YTDLP` and `MUSEFLOW_DENO` to the online helper executables, and optionally `MUSEFLOW_DATA_DIR` to your chosen library directory. Follow the pinned-runtime notes below and [LOCAL-PLAYER.md](LOCAL-PLAYER.md); ensure native runtime dependencies are available. Building does not install or download libmpv/yt-dlp/Deno. Review their licensing before redistributing binaries.

## Maintainer's local storage layout

The following optional helper is specific to the maintainer's G: installations. Do not run it unchanged on a differently configured machine. From the PC workspace in PowerShell:

```powershell
. .\tools\Use-MuseFlowPcEnvironment.ps1
cargo test --locked --offline
cargo clippy --locked --offline --all-targets -- -D warnings
cargo build --locked --offline --release
& "$env:CARGO_TARGET_DIR\release\museflow-pc.exe"
```

Omit `--offline` only when deliberately fetching missing locked dependencies. The helper changes only its process, setting Rust/Cargo/build/cache paths on G:. M: aliases the G:-hosted compiler to avoid GCC path-space failures. Existing unrelated M: drives/mappings are rejected. After compiler processes exit, remove only the owned alias with `subst M: /D`.

Windows resources use GNU `windres` and an application manifest for Common Controls v6, DPI awareness and long paths. build.rs suppresses the pinned GCC's default manifest through an output-directory linker specs file, leaving the compiler installation unchanged. The current resource step does not support MSVC.

The helper defaults `MUSEFLOW_MPV_LIBRARY` to the pinned G:-hosted libmpv DLL and `MUSEFLOW_DATA_DIR` to G:-hosted desktop data. The standalone preview instead finds the adjacent runtime DLL and `portable-data-dir.txt`. Neither requires a Rust compiler at runtime. See [LOCAL-PLAYER.md](LOCAL-PLAYER.md) for path overrides and storage semantics.

## Verification

```powershell
& "$env:CARGO_TARGET_DIR\release\museflow-pc.exe" --demo
& "$env:CARGO_TARGET_DIR\release\museflow-pc.exe" --smoke-test "E:\path\to\isolated-output"
& "$env:CARGO_TARGET_DIR\release\museflow-pc.exe" --data-dir "E:\path\to\fresh-test-data" --import "E:\path\to\generated-fixtures" --null-audio --local-smoke-test "E:\path\to\isolated-output"
```

The local journey expects a fresh test library and at least two generated audio fixtures longer than four seconds; it changes likes/playlists/modes in that test library. Never point it at personal data. Other opt-in journeys cover restore, codec, rapid selection, missing-file, runtime-failure and startup-error scenarios. See the milestone evidence for their controlled preconditions. `--null-audio` decodes without sending audio to speakers, so it cannot certify sound quality or device-loss behavior.

The shell regression mode still uses 10,000 fictional tracks; normal launches start with a real empty library. Runtime validation and distribution license closure remain necessary before a release. The selected libmpv imports `vulkan-1.dll` from the installed graphics runtime; it is available on this host but is not bundled. Windows 10 and clean-machine acceptance remain pending.

## Online verification

The helper also sets MUSEFLOW_YTDLP and MUSEFLOW_DENO to this PC’s pinned G: installations. The standalone preview uses `resolver/yt-dlp.exe` and `javascript/deno.exe` adjacent to its executable. With an explicitly isolated test data directory, `--online-smoke-test <output>` exercises live search/resolution/seek with `--null-audio`. Further opt-in modes cover online-restore, online-page, online-cancel, online-switch (requires imported local fixtures) and online-no-runtime (requires a deliberately missing resolver override). They contact the provider; ordinary unit tests use loopback fixtures and no internet.

## Desktop verification

Slint’s raw-window-handle-06 feature exposes the HWND after the event loop starts. Souvlaki is initialized from that phase, on the UI thread, and is dropped before the window. Desktop native journeys use `--desktop-smoke-test`, `--lyrics-smoke-test` (requires generated matching LRC fixtures), `--media-journey-smoke-test` and `--stream-media-smoke-test`. The latter two are driven by an external fixture-scoped `--media-probe` process through Windows’ media-session API. `--lyrics-provider-probe <output>` saves only counts/status from a known public metadata query. All tests use isolated data and null output. No physical media-key/device test is implied.

## Tray and portable archive

Windows resources now embed file/product versions derived from Cargo.toml. `--tray-lifecycle-smoke-test`, `--tray-exit-smoke-test`, `--tray-restore-smoke-test` and `--tray-fallback-smoke-test` use isolated libraries and native Close requests; fallback uses `--simulate-tray-failure`. `--tray-media-smoke-test` is driven through fixture-scoped Windows media-session commands while hidden. `--auto-exit-after-ms` is an opt-in benchmark helper that allows normal cleanup instead of killing the tray process. See TRAY.md and tools/package_windows.py for archive creation.

## Audio routing validation

See AUDIO-OUTPUT.md for the opt-in routing/stress commands and native configuration probe. Use isolated G: libraries and generated fixtures. These check entry points do not open the normal library unless that path is explicitly supplied.

## Compact layout and keyboard verification

See [USABILITY.md](USABILITY.md) for the fixture-only `--ui-smoke-test` mode, shortcut coverage and renderer-density checks. It requires an explicit isolated `--data-dir` and changes that test library’s preferences. Renderer-density overrides do not certify Windows DPI or screen-reader support.
