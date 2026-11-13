---
type: architecture
status: current
created: 2026-09-07
---

# Artifact acquisition

Resolve exact server coordinates during `init`, pin them in `minegr.toml`, and recreate only that build. Upstream failure never selects a replacement build.

## Resolution

Select Minecraft first from Mojang's manifest, then offer platforms available for that version. The Vanilla version picker keeps only releases in Mojang's newest-first order. It initially shows the five newest releases; typing fuzzy-searches all releases with at most five matches visible.

The Paper version picker flattens the project's version groups, keeps numeric release identifiers, and sorts their components numerically, newest first. It shows five rows and fuzzy-searches the full release list. Build selection remains a separate step.

The Fabric version picker keeps only game versions marked `stable: true`, preserving Fabric's newest-first order. It shows five rows and fuzzy-searches all stable game versions. Loader and Installer selection remain separate steps.

| Platform | Metadata and selection                                                                                                                                                  |
| -------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Vanilla  | `https://piston-meta.mojang.com/mc/game/version_manifest_v2.json`; follow version metadata to `downloads.server.url` and checksum                                       |
| Paper    | `https://fill.papermc.io/v3/projects/paper` and `/versions/{minecraft_version}/builds`; select the newest `STABLE` build and its `downloads["server:default"]` checksum |
| Fabric   | `https://meta.fabricmc.net/v2/versions/game`, `/loader/{minecraft_version}`, and `/installer`; select stable Loader and Installer versions                              |

Paper requests include a descriptive User-Agent with the Minegr version and contact URL or email. Fabric downloads use `https://meta.fabricmc.net/v2/versions/loader/{minecraft_version}/{loader_version}/{installer_version}/server/jar`.

## Installation

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

