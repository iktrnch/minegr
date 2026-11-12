---
type: feature
status: unimplemented
created: 2026-08-24
---

# Status command

`minegr status` prints one daemon snapshot and exits:

```text
Status: running
Memory usage: 2048 MiB
CPU usage: 125.4%
```

States are `starting`, `running`, `stopping`, `failed`, and `stopped`. A missing or unresponsive daemon prints only `Status: stopped`. A retained failed daemon prints `Status: failed` and `Failure: <reason>`.

Show metrics whenever Java is alive. Memory is RSS rounded to whole MiB; CPU has one decimal, with `100%` representing one core. An unavailable metric prints `unavailable` and an explanatory stderr error.

Status is read-only, bypasses Console input, and exits `0` for every reported state, including missing metrics. Protocol or identity mismatch fails. The console uses the same IPC port for state-only subscriptions.

## Related
- [Daemon](../daemon.md)
- [Minegr command](minegr.md)

