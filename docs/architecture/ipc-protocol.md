---
type: architecture
status: current
created: 2026-08-24
---

# IPC protocol

The daemon listens at `$XDG_RUNTIME_DIR/minegr/<uuid>.sock`. Each daemon-backed command opens one full-duplex `UnixStream`. Most sessions use one logical port; the console session carries `Logs`, `Status`, and `Console` together.

## Transport

Authenticate the peer UID, then split the stream into read and write halves. One reader routes requests. Producers send typed messages through a bounded 32-message `mpsc` channel to one writer, which alone serializes and writes frames.

Use Serde's standard compact JSON encoding, prefixed by a four-byte big-endian length. Reject empty payloads and payloads over 16 MiB. Invalid framing, JSON, ordering, or port use returns `ProtocolError` where possible and closes the connection.

```rust
enum Exchange<Req, Res> {
    Request(Req),
    Response(Res),
    Close,
    Closed(CloseReason),
}

enum CloseReason {
    Requested,
    DaemonStopping,
    FellBehind,
}

enum HandshakeResponse {
    Accepted,
    Rejected(HandshakeError),
}

enum Message {
    Handshake(Exchange<HandshakeRequest, HandshakeResponse>),
    Start(Exchange<StartRequest, StartResponse>),
    Stop(Exchange<StopRequest, StopResponse>),
    Logs(Exchange<LogsRequest, LogsResponse>),
    Console(Exchange<ConsoleRequest, ConsoleResponse>),
    Status(Exchange<StatusRequest, StatusResponse>),
    // Other daemon-backed operations.
    ProtocolError(ProtocolError),
}
```

Requests and responses use distinct types. Responses may carry updates followed by one terminal outcome. Wire errors expose stable codes and safe fields; map internal errors at the tunnel boundary. Use typed errors, optionally deriving `thiserror::Error`, rather than `anyhow`.

## Sessions

Every connection starts with the Minegr package version, server UUID, and canonical UTF-8 configuration path. Require an exact version and identity match, including for `stop`. A version rejection reports both versions. There is no separate protocol version or cross-version compatibility guarantee.

The first post-handshake message selects the session. Permit only its port, or the console's three ports. Keep at most one unanswered request per port; log and state subscriptions may coexist with a pending Console input. Route through the connection and port without transport request IDs.

Subscriptions close with `Close` / `Closed(Requested)` or EOF. A full streaming response queue disconnects that client with `FellBehind`; it must never block the daemon. One-shot responses have a five-second per-frame write timeout. Drain responses on graceful completion; discard them on protocol or connection failure.

## Logs and status

Serve log history and live entries as one ordered response, following [logging](logging.md). Target 64 KiB JSON batches; send a larger line alone if it fits the frame limit. Replace an oversized streamed line with a notice directing users to `logs/latest.log`.

Status subscriptions send the initial state and changes. CPU and memory metrics are one-shot data. At final shutdown, deliver observed log lines and the final `stopped` state before `Closed(DaemonStopping)`.

## Console input

Each console connection has a client UUID and increasing `u64` input IDs starting at one. Increment after acknowledgement. The daemon tracks the highest accepted ID per connection and acknowledges older or repeated IDs without enqueueing again. Disconnection clears this state; clients never retry inputs automatically.

All clients share one unbounded Console input FIFO. [Operation coordination](operation-coordination.md) controls consumption and lifecycle priority.

## Shutdown and local access

`StopRequest` has no fields. The first accepted request receives `Requested`, then `Stopped` or `Failed` after coordinated shutdown. During shutdown, new handshakes receive `DaemonStopping`, which later stop clients interpret as `AlreadyInProgress`. Existing tunnels accept no new work. Remove the socket after Java exits and output drains.

- Runtime directory mode: `0700`; socket mode: `0600`; accept only the owner's UID.
- Require `XDG_RUNTIME_DIR`; do not fall back to `/tmp`.
- Allow five seconds for handshake and session setup, no subscription idle timeout, and at most 128 simultaneous connections.
- EOF and write failure detect disconnects; no heartbeat.
- Disconnect semantics follow the command: streams detach, acknowledged inputs remain queued, accepted stop/restart continue, and initial startup is cancelled before readiness.
- `start` removes a stale socket only when it is an owner-matched Unix socket with no responsive daemon.

## Related

- [Daemon](../features/daemon.md)
- [Glossary](../glossary.md)
- [Logs command](../features/commands/logs.md)
- [Console command](../features/commands/console.md)
- [Frame messages over Unix streams](../decisions/0004-frame-messages-over-unix-streams.md)
- [Use one or two IPC tunnels](../decisions/0005-use-one-or-two-ipc-tunnels.md)
- [Sync vs async](../decisions/0002-sync-vs-async.md)
- [Logging](logging.md)
- [Operation coordination](operation-coordination.md)

