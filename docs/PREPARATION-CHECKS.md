# Preparation checks — 14 November 2026

This workspace contains preparation material only. There is no desktop application to compile, launch or benchmark yet.

- 252 source/assets/license/reference files copied verbatim, totaling 14,295,768 bytes (about 13.6 MiB).
- SHA-256 hashes verified against both the destination copies and current original source files. Generated preparation documents/configuration have their own hashes in `COPY-MANIFEST.json`.
- Every workspace file is accounted for by the manifest, except the manifest itself, which deliberately has no circular self-hash.
- Newly authored Markdown links checked. Verbatim Android README/notices retain original Android-relative links, some outside this selected snapshot.
- Exclusion check passed: no Git history, Android build scripts, APKs, binaries, caches, signing material, local configuration or device exports were copied.
- Gitleaks source scan passed: no leaks found. One exact-path/exact-value exception preserves the previously reviewed public InnerTube web-client constant; other secrets remain detectable. The redacted scan report is outside this workspace under `../artifacts/museflow-pc-workspace-preparation-2026-11-14/`.
- The Android working-tree status was unchanged before/after preparation, and copied source hashes still match the originals.
- No tools installed, environment variables changed, Git repository initialized, application code scaffolded or publishing performed.

Android build 66 has user-reported glass bugs; phone investigation and Android development remain paused. No Android test result is presented as desktop acceptance. All startup/RAM targets and Windows/Linux runtime checks remain unmeasured pending implementation.
