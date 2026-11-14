---
type: proposal
status: proposed
created: 2026-09-07
---

# Addon configuration

Basic Modrinth addon installation and pinning are implemented by the [add command](../features/commands/add.md). Portable addon configuration remains deferred beyond that initial support.

Embedded compressed data, external archives, and readable TOML remain alternatives. File ownership, secrets, size limits, and update rules must be designed before adoption.

The current schema stores pinned addon files in `[[addons]]` alongside Minecraft properties. This proposal concerns portable addon settings, dependency management, and further update workflows; it changes no current contract.

## Related

- [Configuration](../architecture/configuration.md)
- [Sync command](../features/commands/sync.md)


