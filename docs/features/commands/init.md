---
type: feature
status: unimplemented
created: 2026-08-24
---

# Init command

`minegr init` creates an instance configuration or materializes missing managed files.

## Options

| Flag | Meaning |
| --- | --- |
| `--name <name>` | Display name; defaults to the root directory name |
| `--minecraft-version <version>` | Exact Minecraft version |
| `--platform <platform>` | `vanilla`, `paper`, or `fabric` |
| `--memory <size>` | Matching `-Xms` and `-Xmx`; default `2G` |
| `--port <port>` | `server-port`; default `25565` |
| `--accept-eula` | Explicit Minecraft EULA acceptance |
| `--yes` | Skip final confirmation, never EULA acceptance |
| `--uuid` | Replace only an existing configuration's UUID |

Flags bypass their prompts; invalid values fail. Missing required answers need an interactive terminal or explicit flags.

`--memory` is an init-only shortcut; edit `minegr.toml` for later Java changes.

## New configuration

1. Resolve the path and gather name, Minecraft version, compatible platform, memory, and properties.
2. Resolve and pin the exact stable build using [artifact acquisition](../../architecture/artifact-acquisition.md).
3. Obtain explicit EULA acceptance and default server arguments to `["nogui"]`.
4. Validate, show a summary, and confirm unless `--yes` is set.
5. Atomically write owner-only `minegr.toml` and materialize `server.jar`, `server.properties`, and `eula.txt`.

Use Clap and Dialoguer to fill `InitConfig`; `create_config` derives, writes, and returns `Config`. Java must already be available and compatible.

## Existing configuration

Validate and create only missing managed files from pinned coordinates. Never overwrite differing files, infer coordinates from a JAR, or import mutable state. A non-empty root is allowed if managed paths do not conflict.

`init --uuid` changes only the UUID and is rejected if the selected instance is running. It may repair a copied configuration whose UUID is running at another path.

On materialization failure, preserve configuration for retry and name any conflicting file to remove. Shared [validation](../../architecture/validation.md) and file-safety rules apply.

On success:

```text
Configuration: <path>
Start it with: minegr start --config <path>
```

## Related

- [Configuration](../../architecture/configuration.md)
- [Artifact acquisition](../../architecture/artifact-acquisition.md)
- [Filesystem layout](../../architecture/filesystem-layout.md)
- [Validation](../../architecture/validation.md)
- [Sync command](sync.md)
- [Start command](start.md)

