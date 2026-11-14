---
type: architecture
status: current
created: 2026-08-28
---

# Configuration

One versioned `minegr.toml` describes an instance's identity, pinned server software, Java launch settings, EULA acceptance, and Minecraft properties. Copying it recreates the configured initial state; worlds and other mutable data are transferred separately.

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

## Schema

- `[minegr]`: format version, UUID, and display name. Require `config_version = 1`.
- `[minecraft]`: Vanilla requires `version`; Paper adds `build`; Fabric adds `loader` and `installer`. Reject fields for another platform. Store an upstream checksum as `<algorithm>:<hex>` when available.
- `[java]`: optional executable, JVM arguments, and server arguments. If omitted, discover compatible Java from `PATH`. Minegr supplies `-jar ./server.jar` and launches without a shell. It never installs Java.
- `[minecraft.properties]`: recognized properties are typed and validated; unknown properties may be strings, numbers, or booleans. Omitted recognized properties use Minecraft defaults when reconstructed.
- `[[addons]]`: optional pinned Modrinth addon records. Each contains the API filename as `name`, the exact `version_id`, an algorithm-qualified `hash` (`sha512:<hex>`), and `version_type` as the most permissive permitted channel for future updates. Missing addon lists default to empty and empty lists are omitted when saving. The channel policy is `release`, `beta` (including release), or `alpha` (including beta and release).
- Reject unknown keys outside the properties table. Values are literal; no environment expansion.

The configuration covers Minecraft properties and pinned addon files. Portable addon settings and platform-specific configuration remain deferred. Mutable state includes worlds, player data, access lists, logs, crash reports, caches, and runtime metadata.

## File operations

The configuration's canonical parent is the server root. A non-empty root is allowed; conflicting managed files are rejected.

`init` creates configuration and missing managed files without overwriting differing files. `init --uuid` changes only identity. `sync` replaces only the properties table from a stopped server's `server.properties`. `add` appends the installed addon's pinned metadata. Other commands never rewrite configuration.

Write to an adjacent temporary file, flush, and rename atomically. Preserve unrelated comments and formatting where practical. Create configuration with mode `0600`; warn about broader permissions and reject foreign-owned or non-regular files. Treat its contents as sensitive.

## Related

- [Add command](../features/commands/add.md)
- [Commands](commands.md)
- [Filesystem layout](filesystem-layout.md)
- [Validation](validation.md)
- [Artifact acquisition](artifact-acquisition.md)
- [Init command](../features/commands/init.md)
- [Sync command](../features/commands/sync.md)
- [Use one declarative instance file](../decisions/0006-use-one-declarative-instance-file.md)

