---
type: architecture
status: current
created: 2026-08-28
---

# Configuration

## Purpose

Define `minegr.toml` as the portable, declarative description of one Minegr-managed Minecraft server instance.

## Responsibilities

- Store Minegr identity, pinned server-core coordinates, Java launch settings, EULA acceptance, and Minecraft properties.
- Recreate the minimum managed files required to launch the same initial server configuration.
- Exclude mutable server state.
- Provide a versioned, strictly validated format.

## Structure

```toml
[minegr]
config_version = 1
uuid = "018f0000-0000-7000-8000-000000000000"
name = "survival"

[minecraft]
platform = "paper"
version = "26.2"
build = 42
checksum = "sha256:<hex>"
eula = true

[java]
executable = "java"
jvm_args = ["-Xms2G", "-Xmx2G"]
server_args = ["nogui"]

[minecraft.properties]
server-port = 25565
white-list = true
```

`[minegr]` contains format version, UUID, display name, and future Minegr-owned settings. `[minecraft]` contains the selected platform and immutable artifact coordinates. Vanilla requires `version`; Paper requires `version` and `build`; Fabric requires `version`, `loader`, and `installer`. Fields for other platforms are rejected. A checksum uses `<algorithm>:<hex>` when upstream publishes one.

`[java]` separates the Java executable, JVM arguments, and server arguments. The executable is optional; omission discovers a compatible `java` from `PATH`. Minegr supplies `-jar ./server.jar` itself and launches Java without a shell. Java is a host prerequisite and Minegr never installs it.

`[minecraft.properties]` maps to `server.properties`. Recognized properties are strongly validated. Unknown properties are allowed as strings, numbers, or booleans for forward compatibility and are preserved when `sync` captures the file. Removing a recognized property from TOML restores its Minecraft default when `server.properties` is next reconstructed.

The initial schema does not contain addons, environment interpolation, or platform-specific configuration files. Those require separate future contracts.

## Data and control flow

`init` creates and reads `minegr.toml`, validates it, resolves the pinned artifact, and materializes missing managed files. It never overwrites a differing managed file. `sync` runs only while the server is stopped and replaces `[minecraft.properties]` with the complete parsed contents of `server.properties`; it never changes any other section.

Normal commands deserialize the selected file with TOML and Serde. Pure validation runs for every Command. Commands do not rewrite the file except `init --uuid` and `sync`.

Writes use a temporary file beside `minegr.toml`, flush it, and rename it atomically. Rewrites preserve comments and formatting outside the changed values on a best-effort basis; correct atomic output takes precedence over presentation preservation.

## Interfaces

### Inputs

- A UTF-8 `minegr.toml` selected by the global `--config` option.
- `server.properties` for `sync`.
- Interactive `init` answers or equivalent command options.

### Outputs

- A validated runtime `Config`.
- Atomically created or updated `minegr.toml`.
- Inputs for artifact acquisition and managed-file generation.

## Invariants

- The canonical parent of `minegr.toml` is the server root.
- `config_version = 1` is required. Other versions are rejected until a migration contract exists.
- Unknown keys are rejected outside `[minecraft.properties]`.
- Values are literal; environment variables are not expanded.
- Exact platform builds are pinned. Recreating an instance never silently selects a newer build.
- `minegr.toml` is created with mode `0600`. Broader permissions warn; a non-regular file or file owned by another user is rejected.
- The file may contain secrets and must be treated as sensitive.
- Worlds, player data, advancements, statistics, bans, operators, whitelist entries, logs, crash reports, caches, and runtime metadata are mutable state and are not represented.
- Configuration may be created in a non-empty root. Validation rejects conflicting managed paths rather than importing or overwriting them.

## Related

- [Commands](commands.md)
- [Filesystem layout](filesystem-layout.md)
- [Validation](validation.md)
- [Artifact acquisition](artifact-acquisition.md)
- [Init command](../features/commands/init.md)
- [Sync command](../features/commands/sync.md)
- [Use one declarative instance file](../decisions/0006-use-one-declarative-instance-file.md)
