---
type: architecture
status: current
created: 2026-09-07
---

# Validation

## Purpose

Define deterministic configuration and host checks shared by Commands.

## Responsibilities

- Separate pure configuration checks from host-dependent checks.
- Report all independent findings in a stable order.
- Prevent destructive adoption, invalid launches, and unsafe file access.

## Structure

Validators implement a shared trait and return structured errors and warnings. Validation is not represented by one mutable validator object. Independent checks run even when another check fails; a check whose prerequisite failed is skipped with one explanation.

Command adapters perform read-only host probes and supply each check as `Passed`, `Failed(reason)`, or `Unavailable(reason)`. The validation layer orders those observations, applies exact check-and-subject prerequisites, and converts failures or unavailable observations into blocking findings. Live discovery and mutation remain outside the side-effect-free validator.

Pure validation covers TOML structure, format version, unknown keys, UUID, platform coordinates, checksums, Java arguments, Minecraft properties, and cross-field compatibility. Host validation covers paths, ownership, permissions, Java compatibility, managed artifacts, memory, disk space, ports, active processes, and server-root conflicts.

## Data and control flow

Every Command performs pure validation. `init`, `start`, and `restart` also run the applicable host checks. `sync` requires the server to be stopped, parses all of `server.properties`, and writes nothing when any property is duplicate or malformed.

Errors block the operation. Warnings are written to stderr and do not block unless a specific risky operation explicitly requires confirmation. Non-interactive callers use the operation's explicit force option rather than treating `--yes` as universal consent.

An active `session.lock` or occupied configured port is evidence of a possibly unmanaged server. Minegr refuses the conflicting operation, never signals an unmanaged process, and reports how to inspect it.

## Interfaces

### Inputs

- Parsed configuration and canonical server-root paths.
- Host files, Java metadata, process indicators, port state, memory, and disk state.

### Outputs

- Ordered structured errors and warnings.
- A validated value for command execution.

## Invariants

- Validation has no side effects.
- Assertions are not weakened because a host check is unavailable.
- Unknown Minegr keys, incompatible Java, checksum mismatch, and conflicting managed files are errors.
- Broader-than-owner configuration permissions are warnings; foreign ownership and non-regular files are errors.
- A manually replaced `server.jar` never changes pinned coordinates automatically.
- `init` names a differing managed file and tells the user to remove it before retrying; it never overwrites it.

## Related

- [Configuration](configuration.md)
- [Filesystem layout](filesystem-layout.md)
- [Artifact acquisition](artifact-acquisition.md)
- [Init command](../features/commands/init.md)
- [Start command](../features/commands/start.md)
