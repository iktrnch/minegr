# Documentation Index

These documents define the planned MVP; the Rust implementation is still a
skeleton. Architecture records own shared rules, and feature records describe
commands and user-visible behaviour. All documented commands remain in scope.

Proposals are deferred ideas, not MVP requirements. ADRs preserve decision
history; ADR 0001 is superseded by ADR 0003. For implementation details, code
and tests remain authoritative.

## General

- [Glossary](glossary.md)

## Agents

- [Domain Docs](agents/domain.md)
- [Issue tracker: Linear](agents/issue-tracker.md)

## Architecture

- [Artifact acquisition](architecture/artifact-acquisition.md)
- [Commands](architecture/commands.md)
- [Configuration](architecture/configuration.md)
- [Filesystem layout](architecture/filesystem-layout.md)
- [IPC protocol](architecture/ipc-protocol.md)
- [Logging](architecture/logging.md)
- [Operation coordination](architecture/operation-coordination.md)
- [Validation](architecture/validation.md)

## Decisions

- [Use Inquire for interactive prompts](decisions/0001-Interactive-prompt-crate.md)
- [0002-sync-vs-async](decisions/0002-sync-vs-async.md)
- [Use Dialoguer for interactive prompts](decisions/0003-use-dialoguer-for-interactive-prompts.md)
- [Frame messages over Unix streams](decisions/0004-frame-messages-over-unix-streams.md)
- [Use one or two IPC tunnels](decisions/0005-use-one-or-two-ipc-tunnels.md)
- [Use one declarative instance file](decisions/0006-use-one-declarative-instance-file.md)
- [Tail Minecraft latest.log](decisions/0007-tail-minecraft-latest-log.md)
- [Coordinate exclusive operations in the daemon](decisions/0008-coordinate-exclusive-operations-in-the-daemon.md)

## Features

- [backup-command](features/commands/backup.md)
- [console-command](features/commands/console.md)
- [init-command](features/commands/init.md)
- [logs-command](features/commands/logs.md)
- [minegr-command](features/commands/minegr.md)
- [restart-command](features/commands/restart.md)
- [start-command](features/commands/start.md)
- [status-command](features/commands/status.md)
- [stop-command](features/commands/stop.md)
- [sync-command](features/commands/sync.md)
- [daemon](features/daemon.md)

## Proposals

- [Addon configuration](proposals/addon-configuration.md)
- [Cross-version stop](proposals/cross-version-stop.md)

