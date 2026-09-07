---
type: architecture
status: current
created: 2026-09-07
---

# Artifact acquisition

## Purpose

Define how Minegr resolves, downloads, verifies, and installs the exact server core pinned by `minegr.toml`.

## Responsibilities

- Resolve eligible Minecraft versions and platform builds from authoritative upstream services.
- Pin exact build coordinates before materialization.
- Verify downloaded artifacts when upstream provides a checksum.
- Publish `server.jar` atomically without a persistent cache.

## Structure

Vanilla is identified by the exact Minecraft version. Paper adds an exact stable build. Fabric adds exact Loader and Installer versions. The configuration stores these coordinates and an upstream checksum when available.

Interactive initialization selects a Minecraft version first from Mojang's version manifest, then offers only platforms available for it. The picker shows 12 entries before scrolling, prioritizes the newest eligible releases, supports search, and reveals snapshots when the search matches snapshot identifiers.

Authoritative endpoints are:

- Vanilla versions: `https://piston-meta.mojang.com/mc/game/version_manifest_v2.json`; follow the selected version metadata to `downloads.server.url` and its checksum.
- Paper versions: `https://fill.papermc.io/v3/projects/paper`; builds: `https://fill.papermc.io/v3/projects/paper/versions/{minecraft_version}/builds`. Select the newest `STABLE` build once and store its ID and `downloads["server:default"]` checksum. Every request uses a descriptive User-Agent with the Minegr version and contact URL or email.
- Fabric games: `https://meta.fabricmc.net/v2/versions/game`; loaders: `https://meta.fabricmc.net/v2/versions/loader/{minecraft_version}`; installers: `https://meta.fabricmc.net/v2/versions/installer`. Select the newest stable Loader and Installer once, then fetch `https://meta.fabricmc.net/v2/versions/loader/{minecraft_version}/{loader_version}/{installer_version}/server/jar`.

Fabric's `server.jar` is a launcher. On first Java start it may download the server and Fabric files for the pinned coordinates. That is Fabric launcher behaviour rather than Minegr selecting or installing a different build; a bootstrap failure is a normal startup failure.

## Data and control flow

1. Resolve the exact coordinates and write them to the validated configuration.
2. Create a unique owner-only directory in the operating system's temporary directory.
3. Download the artifact without using a persistent cache.
4. Verify the upstream checksum when one is available.
5. Copy the verified bytes to a temporary file beside `<server-root>/server.jar`.
6. Flush and atomically rename the file to `server.jar`.
7. Remove temporary data.

An unavailable upstream may be retried but never causes Minegr to select a different build. Failure leaves the valid configuration in place and does not publish a partial artifact.

## Interfaces

### Inputs

- Pinned `[minecraft]` coordinates.
- Mojang, Paper, and Fabric metadata and artifact services.

### Outputs

- A verified `<server-root>/server.jar`.
- Actionable network, availability, or integrity errors.

## Invariants

- Recreation uses the exact pinned build.
- No download is executed through a shell.
- Temporary paths are unique and owner-only.
- A manually replaced `server.jar` that does not match its pinned checksum fails validation.
- Minegr's `start` path never downloads artifacts; it directs the user to rerun `init` when a managed file is missing. A configured Fabric launcher may perform its own first-run bootstrap.
- Java installation is outside artifact acquisition.

## Related

- [Configuration](configuration.md)
- [Filesystem layout](filesystem-layout.md)
- [Validation](validation.md)
- [Init command](../features/commands/init.md)
