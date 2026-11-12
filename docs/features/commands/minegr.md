---
type: feature
status: unimplemented
created: 2026-08-27
---

# Minegr command

`minegr` selects an instance and dispatches a command. Global `--config <path>` defaults to `./minegr.toml` and works before or after the subcommand:

```text
minegr start --config ./servers/survival/minegr.toml
minegr logs --config ./servers/survival/minegr.toml --follow
```

Resolve relative paths from the working directory without searching parents. The canonical UTF-8 configuration path determines the server root. `init` requires an existing parent and may create that exact file; other commands require the file.

## Commands

| Command | Purpose |
| --- | --- |
| [init](init.md) | Create configuration or materialize missing files |
| [sync](sync.md) | Capture stopped-server properties into configuration |
| [start](start.md) | Start Java through its daemon |
| [stop](stop.md) | Stop Java and the daemon |
| [restart](restart.md) | Replace Java while retaining the daemon |
| [status](status.md) | Print state and process usage |
| [logs](logs.md) | Read or follow the current Minecraft log |
| [console](console.md) | Open the interactive console |
| [backup](backup.md) | Archive world files |

Command data uses stdout; progress, warnings, and errors use stderr. Stdout and exit codes support scripting; the TUI does not.

## Exit codes

| Code | Meaning |
| --- | --- |
| `0` | Success, normal stream closure, or client detachment |
| `2` | Usage or configuration error |
| `3` | Server or daemon unavailable |
| `4` | Accepted operation failed |
| `130` | Foreground work interrupted and cancelled |

Clap retains its standard parse-error behaviour. Configuration errors name the path and reason. Define `--config` once with Clap's `global = true`.

## Related

- [Init command](init.md)
- [Sync command](sync.md)
- [Start command](start.md)
- [Commands architecture](../../architecture/commands.md)

