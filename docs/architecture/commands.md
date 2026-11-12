---
type: architecture
status: current
created: 2026-08-27
---

# Commands

Clap parses commands and the global `--config <path>` option, defaulting to `./minegr.toml`. Resolve relative paths from the working directory without searching parents. Canonicalize existing files; for new configuration, canonicalize the existing parent and append the filename. Require a UTF-8 path.

## Dispatch

Keep startup and dispatch in `main.rs`, command handlers in focused modules, and shared terminal operations in a UI module.

Normal handlers receive the canonical path and a deserialized `Config` using TOML and Serde. New-instance `init` fills an `InitConfig` from flags or prompts; `create_config` writes and returns the resulting `Config`. Existing-file `init` loads it to materialize missing files.

Dispatch is synchronous. Daemon and socket operations use Tokio, with one runtime per invocation. Handlers return structured errors; dispatch renders the final error once and selects the [exit code](../features/commands/minegr.md#exit-codes).

## Terminal output

- Send command data to stdout; prompts, progress, warnings, and errors to stderr.
- Wrap Dialoguer and Indicatif in semantic UI operations: prompt, progress, success, warning, and error. Callers do not choose styling. Raw logs, status data, and Ratatui rendering use their own output paths.
- Enable terminal presentation only for a terminal destination and outside CI. Treat `CI=1`, `true`, or `yes` case-insensitively as CI.
- Plain output prints each message once without control sequences. `NO_COLOR` disables color.
- Prompts require terminal stdin and stderr and are disabled in CI. Missing answers produce an error naming the required flags.

Configuration writes and validation follow the shared contracts below.

## Related
- [Configuration](configuration.md)
- [Minegr command](../features/commands/minegr.md)
- [Init command](../features/commands/init.md)
- [Sync command](../features/commands/sync.md)
- [Validation](validation.md)
- [Use Dialoguer for interactive prompts](../decisions/0003-use-dialoguer-for-interactive-prompts.md)
- [Sync vs async](../decisions/0002-sync-vs-async.md)

