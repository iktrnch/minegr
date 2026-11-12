---
type: architecture
status: current
created: 2026-09-07
---

# Validation

Shared validators return structured errors and warnings without side effects. Use a common trait with focused validators, not one mutable validator object.

## Checks

| Stage | Checks |
| --- | --- |
| Every command | TOML schema, version, UUID, platform coordinates, checksum format, Java arguments, Minecraft properties, and cross-field compatibility |
| `init`, `start`, `restart` | Applicable paths, ownership, permissions, Java compatibility, artifacts, memory, disk, ports, active processes, and root conflicts |
| `sync` | Stopped server and a complete, valid `server.properties` |

Report independent findings in a stable order; skip checks whose prerequisites failed. Errors block execution. Warnings use stderr; a risky operation requires its specific confirmation or force option, not blanket `--yes` consent.

Reject incompatible Java, unknown Minegr keys, checksum mismatches, foreign-owned or non-regular configuration, and differing managed files. Broader configuration permissions warn. Name conflicting files and tell users to remove them before retrying `init`; never infer new pins from an existing JAR.

An active world `session.lock` or occupied configured port may indicate an unmanaged server. Reject the conflicting operation, explain how to inspect it, and never signal that process. Unavailable host checks do not imply success.

`sync` rejects duplicate or malformed properties and writes nothing on failure.

## Related

- [Configuration](configuration.md)
- [Filesystem layout](filesystem-layout.md)
- [Artifact acquisition](artifact-acquisition.md)
- [Init command](../features/commands/init.md)
- [Start command](../features/commands/start.md)


