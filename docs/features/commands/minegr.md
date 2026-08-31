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
- [Start command](start.md)
