---
type: feature
status: unimplemented
created: 2026-08-27
related_code: []
---
# minegr-command

## Summary

`minegr` selects a configuration file and dispatches a server-management command.

### Global options

| Option | Description | Mandatory |
| --- | --- | --- |
| `--config <path>` | Direct path to the configuration file. Defaults to `minegr.toml`. | No |

### Subcommands

| Command | Purpose |
| --- | --- |
| `init` | Create or materialize an instance. |
| `sync` | Capture stopped-server properties into `minegr.toml`. |
| `start` | Start Java through the per-server daemon. |
| `stop` | Stop Java and its daemon. |
| `restart` | Replace Java while retaining the daemon. |
| `status` | Print lifecycle state and process usage. |
| `logs` | Read or follow Minecraft's current log through the daemon. |
| `console` | Open the interactive server console. |
| `backup` | Archive mutable world files. |

## Behaviour

All subcommands accept the global option after their name:

```text
minegr start --config ./servers/survival/minegr.toml
minegr logs --config ./servers/survival/minegr.toml --follow
```

Relative paths resolve from the current directory. The configuration file's parent directory is the server root.

## Workflow

1. Parse the command and global configuration path.
2. Resolve the path without searching parent directories.
3. Pass the resolved path to the selected command.

## Rules

- Every subcommand must honor the selected configuration path.
- The resolved canonical configuration path must be valid UTF-8.
- `init` creates that exact file; other commands require it to exist.
- `init` requires the parent directory to exist.
- `sync` is the only Command that reads Minecraft-managed configuration back into `minegr.toml`.
- Stdout data and process exit codes are stable scripting interfaces. Progress, warnings, and diagnostics use stderr. TUI presentation is not a scripting interface.

### Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success, normal stream closure, or successful client detachment. |
| `2` | Usage or configuration error. |
| `3` | Requested server or daemon is unavailable. |
| `4` | An accepted operation failed. |
| `130` | Foreground work was interrupted and cancelled. |

Clap parse errors keep Clap's standard exit behaviour.

## Failure cases

- A configuration file is missing, unreadable, not a regular file, or invalid:

```text
Failed to load configuration <path>: <reason>
```

- The resolved path is not valid UTF-8:

```text
Configuration path is not valid UTF-8: <path>
```

## Implementation

Clap defines `--config` once on the root parser with `global = true`; command-specific options remain on their subcommands.

## Related

- [Init command](init.md)
- [Sync command](sync.md)
- [Start command](start.md)
- [Commands architecture](../../architecture/commands.md)
