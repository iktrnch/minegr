---
type: feature
status: unimplemented
created: 2026-08-24
related_code: []
---

# init-command

## Summary

`minegr init` creates a declarative instance configuration or materializes its missing managed files.

### CLI flags

| Flag | Description | Mandatory |
| --- | --- | --- |
| `--name <name>` | Server display name; defaults to the server-root directory name. | No |
| `--minecraft-version <version>` | Exact Minecraft version. | No |
| `--platform <platform>` | `vanilla`, `paper`, or `fabric`. | No |
| `--memory <size>` | Writes matching `-Xms` and `-Xmx` JVM arguments; defaults to `2G`. | No |
| `--port <port>` | Writes `server-port`; defaults to `25565`. | No |
| `--accept-eula` | Records explicit Minecraft EULA acceptance. | No |
| `--yes` | Skips the final creation confirmation; never implies EULA acceptance. | No |
| `--uuid` | Regenerates only the UUID in an existing configuration. | No |

## Behaviour

Without an existing `minegr.toml`, the command gathers a complete configuration, writes it atomically with mode `0600`, then materializes `server.jar`, `server.properties`, and `eula.txt`.

With an existing configuration, normal `init` validates it and creates only missing managed files. It never overwrites a differing file. This lets a copied `minegr.toml` recreate the configured initial instance without importing or changing mutable state.

`init --uuid` changes only the selected configuration's UUID. It is allowed for a copied configuration whose old UUID belongs to a daemon at another canonical path, but is rejected when the selected instance itself is running.

## Workflow

### Create configuration

1. Resolve the selected configuration path and canonical server root.
2. Prompt for a name when omitted.
3. Select a Minecraft version from Mojang's authoritative list. Show 12 entries before scrolling, support search, prioritize newest eligible releases, and reveal snapshots when search matches them.
4. Show only Vanilla, Paper, or Fabric choices available for that version.
5. Resolve the exact stable platform build and checksum where available.
6. Prompt for explicit EULA acceptance.
7. Convert `--memory <size>` or its `2G` default to `-Xms<size>` and `-Xmx<size>` in `java.jvm_args`; default `java.server_args` to `["nogui"]`.
8. Collect Minecraft properties, including the selected port.
9. Validate all answers, show a summary, and ask for confirmation unless `--yes` was passed.
10. Write `minegr.toml` atomically.
11. Materialize missing managed files and report the next action.

### Materialize existing configuration

1. Load and validate `minegr.toml`.
2. Check every managed path.
3. Create missing files and download only the pinned artifact.
4. Fail when an existing managed file differs.

## Rules

- Java is a host prerequisite. Select a compatible runtime from the optional configured executable or `PATH`; never install Java.
- The configuration remains after a materialization failure so the command can be retried.
- Downloads use unique owner-only system temporary storage and no persistent cache.
- A non-empty server root is allowed, but conflicting managed files fail validation.
- The command never guesses coordinates from an existing `server.jar` and never imports mutable state.
- `--memory` exists only on `init`; later Java changes are made in `minegr.toml`.
- Values provided by flags skip their corresponding prompts. Invalid flag values fail instead of prompting again.

## Failure cases

- Required prompts are unavailable because stdin or stderr is not a terminal.
- EULA acceptance or final confirmation is denied.
- The selected version and platform have no eligible artifact.
- Upstream metadata or downloads are unavailable.
- Java is incompatible or unavailable.
- Configuration, paths, permissions, port, memory, disk, or checksum validation fails.
- A managed file exists with content that differs from the configuration. Name it and tell the user to remove it before retrying.
- The selected instance is running during `init --uuid`.

## Implementation

Clap populates an `InitConfig` questionnaire. `create_config(InitConfig)` derives the runtime `Config`, writes it, and returns it for validation. Normal existing-file initialization loads `Config` and materializes missing files. Artifact and validation details remain in their architecture documents.

On success, print:

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
