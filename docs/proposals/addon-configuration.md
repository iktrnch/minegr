---
type: proposal
status: proposed
created: 2026-09-07
---

# Addon configuration

## Problem

A future Minegr instance definition must recreate plugins and mods with their configuration without turning mutable server state into configuration.

## Proposal

Add addon entries to `minegr.toml` after plugin and mod management is designed. An entry may carry compressed, base64-encoded configuration content so the single instance file remains portable.

## Motivation

Plugin and mod configuration is often required to reproduce useful server behaviour, but its file boundaries, secrets, and merge rules are not part of the initial Minecraft-only configuration contract.

## Possible approach

Use `[[addons]]` entries with exact addon coordinates and an explicitly bounded archive of relative configuration files. Decode only beneath approved addon paths and verify all sizes and checksums before publication.

## Alternatives

- Keep addon configuration outside `minegr.toml`.
- Reference external archives rather than embedding data.
- Use readable nested TOML for addons with known schemas.

## Open questions

- Which files belong to each addon and who declares them?
- What compressed and decoded size limits apply?
- How are secrets, binary files, updates, and merge conflicts handled?
- Which platform and addon versions may reuse an embedded configuration?

## Impact

This would extend configuration validation, artifact acquisition, filesystem ownership, `init`, and `sync`. It is not part of the current schema.

## Related

- [Configuration](../architecture/configuration.md)
- [Sync command](../features/commands/sync.md)

