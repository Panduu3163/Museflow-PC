# Environment and storage

Workspace: `E:\ALL Project & Idea Workspace\WSL BACKUP\Museflow PC version`.

All source/reference work stays here on E:. The Android source remains separately in `museflow-upgrade` and is unchanged. The desktop player now has a Cargo project and a G:-hosted Rust GNU toolchain. See [BUILDING.md](BUILDING.md) for exact versions and commands.

Storage allocation (the original preparation installed nothing; the later shell milestone installed only Rust/Cargo and the portable compiler):

| Purpose | Proposed path |
| --- | --- |
| Rust toolchains / RUSTUP_HOME | `G:\Essential Softwares\Rust\Rustup` |
| Cargo tools/cache / CARGO_HOME | `G:\Essential Softwares\Rust\Cargo` |
| Compilation / CARGO_TARGET_DIR | `G:\Essential Softwares\MuseFlow-PC\Build` |
| Portable GNU C compiler/linker | `G:\Essential Softwares\MuseFlow-PC\MinGW` |
| Native audio runtime | `G:\Essential Softwares\MuseFlow-PC\libmpv` |
| Resolver | `G:\Essential Softwares\MuseFlow-PC\yt-dlp` |
| Supported JS runtime | `G:\Essential Softwares\MuseFlow-PC\JS-Runtime` |
| Windows compiler/build tools | `G:\Essential Softwares\Microsoft-Build-Tools` |
| Installer/dependency download cache | `G:\Essential Softwares\MuseFlow-PC\Downloads` |

Rust 1.100.0, its rustfmt/Clippy components and portable WinLibs MinGW were installed on G:. Slint 1.18.1 and other Rust dependencies are in the G: Cargo cache. Build environment variables are process-scoped; machine/user PATH and registry were not changed. M: is a compiler alias to G:, not another installation location. No Microsoft build-tools installation or elevation was used. The pinned libmpv runtime is installed on G:; yt-dlp 2026.10.26 and Deno 2.9.8 are now installed in separate G:-hosted folders for the online milestone. Re-check Windows 10 compatibility and native artifact requirements at those milestones.

No Node.js, JVM or Android SDK is planned as an application runtime requirement. yt-dlp and its supported JS runtime are explicit native helper dependencies and must be accounted for in packaging, licensing and memory measurements. Existing development tools on this PC are not automatically dependencies of the distributed app.

The local preview stores library/settings under `G:\Essential Softwares\MuseFlow-PC\Data`, selected through its adjacent portable marker. Developer/test data paths can be overridden explicitly. Installer customization remains later work; never store personal data in reference material or version control. The G: paths above are this development machine's layout, not hardcoded end-user paths.

The current standalone preview is under `G:\Essential Softwares\MuseFlow-PC\Preview\0.5.0-tray`; libmpv is under `G:\Essential Softwares\MuseFlow-PC\libmpv\20261111`. Test fixtures/evidence remain separately under E:-hosted artifacts. No separate FFmpeg CLI, Visual Studio or GPU driver was installed during this milestone.

Online helper scratch directories are created under the resolver installation’s `temp/` folder on G:, with child-only TEMP/TMP/DENO_DIR overrides. Each task removes only its owned scratch directory after the process job exits. No system/user environment variables are changed.

The 0.4 desktop milestone adds only pinned Rust dependencies to the existing G: Cargo cache; Windows media control APIs are supplied by the OS. Lyrics cache is bounded in memory for the session and creates no new database schema.

Portable ZIP output and isolated extraction tests stay in separate folders under `G:\Essential Softwares\MuseFlow-PC`. The source packager stays on E:. The environment helper points optional Python bytecode caches to G:, with no global environment changes.
