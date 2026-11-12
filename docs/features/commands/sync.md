---
type: feature
status: unimplemented
created: 2026-09-07
---

# Sync command

`minegr sync` captures a stopped server's `server.properties` into `minegr.toml`.

1. Load and validate configuration; reject a running daemon or unmanaged server.
2. Parse the complete properties file, rejecting missing, unreadable, duplicate, or malformed data.
3. Convert known properties to typed TOML values and retain unknown values as strings.
4. Replace the complete `[minecraft.properties]` table and write atomically.

Print `Configuration synchronized: <path>` on success.

Replacement reflects deleted properties. Preserve every other section, including identity, artifact pins, Java settings, and EULA acceptance. Preserve unrelated formatting where practical using a document-preserving TOML editor.

Synchronization is one-way into TOML and requires no confirmation. It never applies TOML to generated files or writes a partial table on failure.

## Related

- [Configuration](../../architecture/configuration.md)
- [Filesystem layout](../../architecture/filesystem-layout.md)
- [Validation](../../architecture/validation.md)
- [Init command](init.md)
- [Addon configuration proposal](../../proposals/addon-configuration.md)


