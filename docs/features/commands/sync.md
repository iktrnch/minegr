---
type: feature
status: unimplemented
created: 2026-09-07
related_code: []
---

# sync-command

## Summary

`minegr sync` captures the stopped server's persisted Minecraft properties into `minegr.toml`.

## Behaviour

The command reads `server.properties`, replaces the complete `[minecraft.properties]` table, preserves every other configuration section, writes the result atomically, and prints:

```text
Configuration synchronized: <path>
```

It is one-way from the Minecraft-managed file into `minegr.toml`. It does not apply TOML to generated files.

## Workflow

1. Load and validate `minegr.toml`.
2. Verify that neither a matching daemon nor an unmanaged Minecraft server is running.
3. Parse all of `server.properties`.
4. Convert recognized properties to their typed TOML values and retain unknown values as strings.
5. Replace `[minecraft.properties]` and atomically rewrite the configuration.

## Rules

- Initially, `server.properties` is the only synchronization source.
- Deletions in `server.properties` are reflected by replacing rather than merging the TOML table.
- Minegr identity, artifact pins, Java arguments, and EULA acceptance are never inferred or changed.
- The server must be stopped, even though the command only reads Minecraft files.
- Preserve comments and formatting outside the changed table on a best-effort basis; correctness and atomicity take precedence.
- No confirmation is required because updating the configuration is the command's sole purpose.

## Failure cases

- The daemon or an unmanaged server appears to be running.
- `server.properties` is missing, unreadable, duplicate, or malformed.
- The configuration cannot be parsed, validated, safely rewritten, or atomically replaced.

No partial property table is written on failure.

## Implementation

Use a document-preserving TOML editor to change only `[minecraft.properties]`. Parse before creating the adjacent temporary output file, then flush and rename it.

## Related

- [Configuration](../../architecture/configuration.md)
- [Filesystem layout](../../architecture/filesystem-layout.md)
- [Validation](../../architecture/validation.md)
- [Init command](init.md)
- [Addon configuration proposal](../../proposals/addon-configuration.md)

