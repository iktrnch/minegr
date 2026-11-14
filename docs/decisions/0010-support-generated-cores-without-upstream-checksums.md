---
type: decision
status: accepted
created: 2026-11-14
supersedes: ["0009"]
superseded_by: []
---

# Support generated cores without upstream checksums

## Context

The shared core installer introduced in ADR 0009 assumes every provider supplies an upstream checksum and byte count. Fabric generates its launcher JAR from Minecraft, loader, and installer versions without publishing a checksum or necessarily sending content length.

Artifact acquisition requires verification when an upstream checksum is available. Fabric must remain distinguishable from the checksum-verified Vanilla and Paper paths.

## Options considered

### Require a checksum for every provider

Reject generated launchers without an upstream checksum.

**Advantages**

- Every installed artifact has an upstream checksum comparison.

**Disadvantages**

- Fabric's generated launcher cannot be installed through the requested API.

### Represent absent upstream metadata explicitly

Keep the provider-to-enum handoff and use an optional checksum and optional advertised download length.

**Advantages**

- All providers use the same streaming and publication implementation.
- Available upstream checksums remain mandatory for Vanilla and Paper.
- Progress can report downloaded bytes accurately without an advertised total.

**Disadvantages**

- Fabric launcher installation has no upstream checksum verification.

## Decision

Retain ADR 0009's shared downloader, owned temporary-file handoff, and enum-owned publication. Change `CoreDownload::checksum` to `Option<CoreChecksum>` and record the actual downloaded byte count. The downloader checks an advertised size from provider metadata or HTTP content length when present.

Fabric resolves the first stable loader for the requested Minecraft version, then the first stable installer. It downloads the generated launcher over HTTPS and returns no upstream checksum. The enum skips hashing for an absent checksum, while preserving checksum verification for Vanilla and Paper. It retains synchronized, non-overwriting publication and temporary-file cleanup for every provider.

Unknown-length downloads display a byte counter and rate. Hash progress is shown only when an upstream checksum can be checked. A digest calculated solely from downloaded bytes is not presented as upstream verification.

## Rationale

The data model represents upstream capabilities directly and preserves the existing file-safety behavior without inventing checksum metadata.

## Consequences

### Positive

- Fabric is supported without duplicating the download or installation pipeline.
- Callers through `Platform::get_core` retain their existing interface.

### Negative

- Direct provider callers must handle the optional checksum.
- Reproducibility still requires persisting the resolved loader and installer coordinates in configuration.

## Related

- [Share core verification and installation](0009-share-core-verification-and-installation.md)
- [Artifact acquisition](../architecture/artifact-acquisition.md)
- [Filesystem layout](../architecture/filesystem-layout.md)
- [Init command](../features/commands/init.md)
