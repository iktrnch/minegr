---
type: feature
status: implemented
created: 2026-11-14
---

# Add command

`minegr add [CONFIG_DIR] <SLUG>` installs a Modrinth plugin or mod and records the exact file in `minegr.toml`. The directory defaults to `.`.

```text
minegr add ./server simple-voice-chat --channel release
minegr add simple-voice-chat
```

## Behaviour

`--channel` / `-c` sets the most permissive permitted release channel. `release` accepts only releases, `beta` accepts beta and release versions, and `alpha` accepts all three. The default is alpha.

Versions must match both the configured Minecraft version and a compatible loader. Fabric accepts Fabric versions; Paper accepts Paper, Spigot, and Bukkit versions. The newest matching publication is selected by its parsed timestamp, with version ID breaking ties deterministically.

The primary file is selected, falling back to the first file when no primary is marked. Its filename must name a JAR directly inside the addon directory. Installation targets `mods/` for Fabric and `plugins/` for Paper. Vanilla rejects addons.

The command displays metadata progress and separate downloading, hashing, and copying progress bars. It downloads into private temporary storage, checks the API byte count and SHA-512, then stages an adjacent temporary file and publishes without overwriting an existing path.

After installation, the command appends an `[[addons]]` record and saves the configuration. The record contains the API filename as `name`, the exact `version_id`, an algorithm-qualified `hash`, and the allowed channel as `version_type`. The saved channel is an update policy and can differ from the installed version's actual channel. Other API metadata is fetched as needed rather than duplicated in configuration.

## Rules

Missing compatible versions, invalid filenames or hashes, HTTP errors, size or hash mismatches, and existing destination paths fail with contextual errors. Already configured filenames or version IDs are rejected. Failed downloads and staging operations discard temporary files.

If the JAR was published but directory synchronization or configuration saving fails, the error identifies that installation already happened. The installed file is retained; resolve the error before retrying, because installation will not overwrite it.

Dependency installation, automatic updates, and restoration of missing pinned addons are not part of this command. A pinned hash verifies downloaded bytes but cannot recover files deleted upstream without a retained copy.

## Related

- [Configuration](../../architecture/configuration.md)
- [Artifact acquisition](../../architecture/artifact-acquisition.md)
- [Addon configuration proposal](../../proposals/addon-configuration.md)
