# Contributing

MuseFlow PC is an ongoing preview. Please open an issue before starting a large change so its scope fits the current [plan](docs/PLAN.md).

For bugs, include the preview version, Windows version, local/online source, reproduction steps and expected/actual behavior. Redact personal paths, stream URLs, cookies and credentials from diagnostics. Do not upload private music files, account data or signing material.

Use the documented GNU [build environment](docs/BUILDING.md). Before proposing code changes, run formatting, the locked unit tests and Clippy with warnings denied. Describe what was tested and what still needs a physical-device or clean-machine check. A successful build alone does not prove playback quality or accessibility.

Preserve GPL licensing, upstream notices and source attribution. Keep binaries, toolchains, caches and personal libraries out of source control. Reference snapshots and their original copy manifest should remain intact; put desktop implementation in `src/` and `ui/`.
