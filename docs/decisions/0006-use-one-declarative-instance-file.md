---
type: decision
status: accepted
created: 2026-09-07
supersedes: []
superseded_by: []
---

# Use one declarative instance file

## Context

Minegr must recreate a server's configured initial state without treating mutable Minecraft data as configuration. Existing commands need one authoritative definition for identity, core artifacts, Java launch options, EULA acceptance, and `server.properties`.

## Options considered

### One declarative `minegr.toml`

Store portable intent in one versioned file and generate the managed runtime files from it.

**Advantages**

- One file can be copied, reviewed, and version-controlled.
- Exact server builds and launch settings are explicit.
- Mutable state remains outside configuration.

**Disadvantages**

- Generated files can drift after initialization.
- Synchronizing on-disk properties back into TOML needs an explicit operation.

### Treat the server directory as configuration

Recreate an instance by copying all configuration-like files from the server root.

**Advantages**

- Preserves upstream and platform files without translation.

**Disadvantages**

- Mixes declarative settings, generated data, secrets, and mutable state.
- Cannot provide one portable source of truth.

## Decision

Use one versioned `minegr.toml` as the declarative instance definition. It includes Minegr identity, exact core coordinates, Java launch configuration, EULA acceptance, and Minecraft properties. It excludes mutable state.

`init` materializes missing managed files. `sync` explicitly captures `server.properties` back into the TOML file while the server is stopped.

## Rationale

The model provides Docker-Compose-like recreation while retaining Minecraft's conventional runtime layout and making state boundaries explicit.

## Consequences

### Positive

- A server's configured initial state is portable and reviewable.
- Artifact upgrades cannot happen implicitly during recreation.
- File drift is visible and handled explicitly.

### Negative

- Platform and addon configuration need later extensions.
- Some TOML rewrites may normalize comments or formatting.
- Users remain responsible for transferring mutable state separately.

## Related

- [Configuration](../architecture/configuration.md)
- [Filesystem layout](../architecture/filesystem-layout.md)
- [Init command](../features/commands/init.md)
- [Sync command](../features/commands/sync.md)

