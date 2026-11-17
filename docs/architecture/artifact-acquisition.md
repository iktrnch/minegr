---
type: architecture
status: current
created: 2026-09-07
---

# Artifact acquisition

Resolve exact server coordinates during `init`, pin them in `moxum.toml`, and recreate only that build. Upstream failure never selects a replacement build.

## Resolution

`Platform::select_version(&self)` delegates to the selected platform's provider. Paper, Fabric, and Vanilla implement the `PlatformProvider` trait with associated functions, so version selection needs no provider instance.

Select Minecraft first from Mojang's manifest, then offer platforms available for that version. The Vanilla version picker keeps only releases in Mojang's newest-first order. It initially shows the five newest releases; typing fuzzy-searches all releases with at most five matches visible.

The Paper version picker flattens the project's version groups, keeps numeric release identifiers, and sorts their components numerically, newest first. It shows five rows and fuzzy-searches the full release list. Core resolution uses the first build in the builds endpoint's returned array, without filtering or reordering it.

The Fabric version picker keeps only game versions marked `stable: true`, preserving Fabric's newest-first order. It shows five rows and fuzzy-searches all stable game versions. Core resolution takes the first entry with `loader.stable = true` from `/loader/{minecraft_version}`, then the first entry with `stable = true` from `/installer`. It uses those exact versions in the generated launcher JAR URL; no unstable fallback is selected.

| Platform | Metadata and selection                                                                                                                                                |
| -------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Vanilla  | `https://piston-meta.mojang.com/mc/game/version_manifest_v2.json`; follow version metadata to `downloads.server.url` and checksum                                     |
| Paper    | `https://fill.papermc.io/v3/projects/paper` and `/versions/{minecraft_version}/builds`; take the first build's `downloads["server:default"]` URL and SHA-256 checksum |
| Fabric   | `https://meta.fabricmc.net/v2/versions/game`, `/loader/{minecraft_version}`, and `/installer`; select stable Loader and Installer versions                            |

Paper requests include a descriptive User-Agent with the Moxum version and contact URL or email. Fabric downloads use `https://meta.fabricmc.net/v2/versions/loader/{minecraft_version}/{loader_version}/{installer_version}/server/jar`.

## Installation

`PlatformProvider::get_core(version)` resolves platform metadata and returns a `CoreDownload` containing an owned private temporary file, the downloaded byte count, and an optional upstream checksum with its algorithm. All providers use the shared download implementation. Vanilla follows Mojang's manifest and version metadata and supplies SHA-1; Paper takes the first returned build's `downloads["server:default"]` and supplies SHA-256. Unknown versions, empty build lists, missing stable Fabric loaders or installers, missing server downloads, failed requests, and size mismatches return contextual errors.

Fabric's generated launcher endpoint does not publish an upstream checksum. The Fabric provider returns no checksum, so installation skips hash verification for that launcher. Advertised sizes are checked against provider metadata or HTTP content length when available; otherwise the downloader records the actual byte count. HTTPS transport and successful streaming do not constitute upstream checksum verification.

`Platform::get_core(version, config_dir)` verifies a supplied checksum, then synchronizes and publishes the JAR as `config_dir/server.jar` through an adjacent temporary file without replacing any existing path. The caller supplies the server root explicitly; relative paths resolve from the current working directory. Direct calls to provider methods only download; they do not verify checksums or install files. See [supporting generated cores without upstream checksums](../decisions/0010-support-generated-cores-without-upstream-checksums.md).

Installation displays separate byte-based progress bars for downloading, hashing when an upstream checksum exists, and copying. Known totals show transfer rates and estimated remaining time. Downloads without an advertised length show a byte counter and rate without a percentage or ETA. A stage is marked complete after its checks pass; copying completes after publication and synchronization. Errors mark the active stage as failed. Progress is drawn on stderr in an interactive terminal and hidden when stderr is redirected.

1. Validate and save the exact coordinates.
2. Download into unique owner-only system temporary storage, without a persistent cache or shell.
3. Verify the upstream checksum when available.
4. Copy to a temporary file beside `server.jar`, flush, and rename atomically.
5. Remove temporary data.

Failure preserves the configuration for retry and publishes no partial artifact. A replacement JAR that differs from its pinned checksum fails validation.

`start` never downloads missing managed files; it directs users to rerun `init`. Fabric's launcher may perform its own first-run bootstrap for the pinned coordinates. Java installation remains the user's responsibility.

## Related

- [Configuration](configuration.md)
- [Filesystem layout](filesystem-layout.md)
- [Validation](validation.md)
- [Init command](../features/commands/init.md)
