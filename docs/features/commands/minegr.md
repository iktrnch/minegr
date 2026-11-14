---
type: feature
status: unimplemented
created: 2026-08-27
---

# Minegr command

`minegr` selects an instance and dispatches a command. A global optional positional `[CONFIG_DIR]` defaults to the current directory and can appear before or after any command:

```text
minegr start ./servers/survival
minegr logs ./servers/survival --follow
```

Resolve relative directories from the working directory without searching parents. The canonical UTF-8 directory is the server root, and its configuration is always `minegr.toml`. `init` requires an existing directory and may create that file; other commands require the file.

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

Clap retains its standard parse-error behaviour. Configuration errors name the path and reason. The former `--config` and `-c` flags are no longer accepted.

## Related

- [Init command](init.md)
- [Sync command](sync.md)
- [Start command](start.md)
- [Commands architecture](../../architecture/commands.md)

