# Reference snapshot and provenance

Prepared 14 November 2026 from the actual on-disk Android workspace, including selected uncommitted changes. `COPY-MANIFEST.json` records the source root, Git HEAD/branch, dirty-state flag, original relative paths, destination paths, sizes, source modification times and SHA-256 hashes. HEAD identifies ancestry only; it does not identify all copied bytes. Individual hashes are authoritative for this snapshot.

## Included

- InnerTube Kotlin catalog/client/model source and unit tests, preserving GPL/upstream attribution.
- An explicit selection of top-level app models, providers, repositories, queue/playback coordination, lyrics parsing/matching and local/portable storage behavior. Android platform adapters occur here as boundary references, not reusable Rust code.
- Related behavior and recovery tests, including backup schema validation. These tests need Android/JVM infrastructure and cannot be run directly in this desktop preparation folder. Android backup schemas are reference for later optional interoperability; Android settings/URIs are not automatically desktop-compatible.
- Project GPL license, historical README/changelog/notices, font notices, existing branding, historical screenshots and theme tokens.

## Excluded

No `.git` history, build scripts/configuration, APKs, outputs/caches, local environment files, signing material, personal music/backup data, private research/handoff notes or phone captures. The new root `.gitignore` and secret-scan policy are preparation safeguards, not a Git repository initialization.

Build 66's glass work was reported buggy by the user and physical investigation is paused. Copied settings/models may name Glass, but no glass/navigation UI implementation has been adopted. Original comments and historical verification statements refer to the Android app at their stated checkpoints, not a desktop test result.

The source client contains a reviewed public web-client API constant; it is not a developer-owned secret or a guaranteed supported service contract. The secret scan uses one exact-value, exact-path exception for that existing constant. Do not import accounts, cookies or user credentials into the snapshot.

## Future porting rules

Port behavior into Rust with fresh fixtures/tests, preserving source-level attribution where adapting implementation. Replace Android services/permissions/Room/DataStore/MediaStore with desktop equivalents. Do not treat a referenced source comment or a passing Android unit test as proof that a provider still works or a Windows journey is correct.

Generated preparation documents are listed with hashes in the manifest separately from verbatim copies. The manifest excludes its own hash to avoid a circular definition. Any new work changes the snapshot and should be distinguished from the preparation baseline.
