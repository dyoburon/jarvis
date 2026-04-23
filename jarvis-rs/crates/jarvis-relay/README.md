# jarvis-relay

WebSocket relay for Jarvis: **bridge** sessions (desktop ↔ mobile PTY) and **broadcast** sessions (host ↔ spectators). The relay parses only the first frame (hello); after that it forwards **opaque text** between peers.

## Wire protocol (relay layer)

Defined in [`src/protocol.rs`](src/protocol.rs).

**Client → relay (first message only):**

| `type`             | Role      | Purpose                          |
|--------------------|-----------|----------------------------------|
| `desktop_hello`    | Desktop   | Creates / rejoins bridge session |
| `mobile_hello`     | Mobile    | Joins existing bridge session    |
| `host_hello`       | Host      | Broadcast session                |
| `spectator_hello`  | Spectator | Watch broadcast                  |

**Relay → client:**

| `type`               | Meaning                          |
|----------------------|----------------------------------|
| `session_ready`      | Registered; includes `session_id` |
| `peer_connected`     | Bridge peer joined               |
| `peer_disconnected`  | Bridge peer left                 |
| `host_connected`     | (Broadcast) host online          |
| `host_disconnected`  | (Broadcast) host left            |
| `viewer_count`       | Spectator count                  |
| `error`              | Fatal / validation error         |

## PTY / encryption (opaque to relay)

After registration, clients exchange JSON **envelopes** (`key_exchange`, `plaintext`, `encrypted`) and inner PTY messages. The relay does not parse those.

- **Desktop:** [`jarvis-app` `relay_client.rs`](../jarvis-app/src/app_state/ws_server/relay_client.rs), [`relay_protocol.rs`](../jarvis-app/src/app_state/ws_server/relay_protocol.rs)
- **Mobile:** [`jarvis-mobile/lib/relay-connection.ts`](../../../jarvis-mobile/lib/relay-connection.ts)

## Keepalive

Mobile sends periodic JSON `{"type":"ping"}` on the WebSocket. The relay **does not forward** these over bridge links (desktop never used them); WebSocket **Ping** frames are still answered with **Pong** as usual.

## Conformance

`session_ready` (and future relay control messages) are covered by shared JSON under [`jarvis-rs/testdata/relay/`](../../testdata/relay/); `jarvis-relay` and `jarvis-app` tests deserialize or serialize against the same file.

## Run

```bash
cargo run -p jarvis-relay -- --help
```

Options and defaults are in `src/main.rs` (listen address, TLS, session limits, etc.).
