---
type: feature
status: unimplemented
created: 2026-08-24
related_code: []
---
# backup-command

## Summary

`minegr backup` creates a ZIP archive of the server's world folders while the server is stopped or running through minegr.
### CLI flags

| Flag | Description | Mandatory |
| --- | --- | --- |
| `--config <path>` | Path to `minegr.toml`. Defaults to `./minegr.toml`. | No |

## Behaviour
Prints `Creating backup…`, creates `backups/backup-DD-MM-YYYY-HH-MM.zip`, then prints `Backup created: <path>`.

The timestamp uses local time. Existing backups are never overwritten or removed automatically.
## Workflow
1. Resolve the server root from the canonical configuration path and acquire its backup lock.
2. Read `level-name` from `server.properties`, defaulting to `world`. Include that folder and any matching `_nether` and `_the_end` folders.
3. Validate paths, reject a duplicate filename, and ensure the available space can hold the included data.
4. If the daemon is running, request a live snapshot. Otherwise, verify that no world `session.lock` is active.
5. Create a temporary ZIP, rename it atomically, and report its path.

For a live snapshot, the daemon runs `save-off` and `save-all flush`, waiting up to five minutes. Console commands are queued until the backup client finishes or disconnects. The daemon then runs `save-on` before executing every queued command.
## Rules
- Only one backup may run per server.
- Preserve world directory names at the ZIP root.
- Include all world data except `session.lock`.
- Symlinks may resolve only within the canonical server root.
- Create `backups/` for its owner only and archives with mode `0600`.
- A stopped or daemon-managed running server may be backed up; an unmanaged running server may not.
## Failure cases
- Configuration, `server.properties`, or world paths are invalid.
- A backup with the same timestamp exists or another backup is active.
- Available space is smaller than the estimated input size.
- The live save fails or exceeds five minutes.
- An active `session.lock` belongs to a server not managed by the daemon.
- Archiving is interrupted or fails.

Failure removes the temporary ZIP. A live backup always restores saving and drains queued console commands before returning.
## Implementation
The CLI owns path validation, size estimation, and ZIP creation. The daemon only controls the live snapshot window and recovers it if the client disconnects.
## Related
- [daemon](daemon.md)
- [restore-command](restore-command.md)
