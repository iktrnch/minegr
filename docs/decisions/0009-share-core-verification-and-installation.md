---
type: decision
status: superseded
created: 2026-11-14
supersedes: []
superseded_by: ["0010"]
---

# Share core verification and installation

## Context

Vanilla and Paper resolve different metadata and use different upstream checksum algorithms. Their downloads need the same streaming, progress reporting, integrity checks, temporary-file cleanup, and publication rules.

## Options considered

### Complete installation in each provider

Each provider downloads, verifies, and publishes its own server JAR.

**Advantages**

- Provider code contains its complete operation.

**Disadvantages**

- File-safety and progress behavior are duplicated across providers.
- Fixes can leave providers with inconsistent installation behavior.

### Return a downloaded artifact to the Platform enum

Providers resolve metadata and use a shared downloader to return an owned temporary file, expected byte count, and algorithm-specific checksum. The enum verifies and installs it.

**Advantages**

- Downloading, hashing, progress reporting, and publication each have one implementation.
- The trait enforces the same provider contract.

**Disadvantages**

- A provider download is not an installed or checksum-verified artifact.
- The shared installer must support each upstream checksum algorithm.

## Decision

`PlatformProvider::get_core(version)` returns `CoreDownload`. The shared downloader streams into private temporary storage and checks the expected size. `Platform::get_core(version)` retains its `Result<()>` interface and owns checksum verification and publication as `server.jar`.

`CoreChecksum` distinguishes Vanilla's SHA-1 from Paper's SHA-256. Publication uses a synchronized adjacent temporary file and never overwrites an existing path. Failed publication discards the staged file before propagating the error.

## Rationale

Platform-specific metadata remains in provider modules while every platform uses the same installation and file-safety behavior.

## Consequences

### Positive

- Existing callers through the `Platform` enum keep the same interface.
- All supported platforms retain downloading, hashing, and copying progress bars.

### Negative

- Direct provider callers must account for the new return type and use the enum to install an artifact.

## Related

- [Artifact acquisition](../architecture/artifact-acquisition.md)
- [Filesystem layout](../architecture/filesystem-layout.md)
- [Init command](../features/commands/init.md)
