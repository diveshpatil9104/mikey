# Wire Protocol

## Framing

Every frame on every transport:

```
┌──────────┬────────────────────┬─────────────────┐
│ 1 byte   │ 4 bytes (u32 BE)   │ N bytes         │
│ type     │ payload length     │ payload         │
└──────────┴────────────────────┴─────────────────┘
```

Receivers **reject any length > 4 MiB** and close the connection (protects against garbage or malicious input).

## Frame types

| Type | Name | Direction | Payload |
|---|---|---|---|
| `0x00` | HELLO | phone → PC | JSON: `proto`, `device_id`, `device_name`, `level` (1–4), `token?`, `resume?`, `caps` |
| `0x10` | WELCOME | PC → phone | JSON: `pc_id`, `pc_name`, `token` (new or same), `resumed`, `pc_caps` (e.g. `aec`, `rnnoise`, `vcam`) |
| `0x11` | PENDING | PC → phone | empty — waiting for user approval (phone dot turns amber) |
| `0x12` | REJECT | PC → phone | JSON: `reason` (`denied`, `busy`, `version`, `bad_token`) |
| `0x01` | AUDIO | phone → PC | media header + PCM s16le **or** Opus packet |
| `0x02` | VIDEO | phone → PC | media header + one JPEG image |
| `0x03` | HEARTBEAT | both | 8-byte send timestamp (µs) — echoed by PC for RTT |
| `0x04` | CONTROL | both | JSON, e.g. `{"audio":{"ns":true,"ns_strength":0.8,"aec":true,"gate_db":-45}}`, `{"video":{"on":true,"w":1280,"h":720}}` |
| `0x05` | BYE | both | JSON: `reason` — clean shutdown, no timeout wait |

### JSON field formats

- All JSON payloads are UTF-8.
- `proto`: integer, the major protocol version. Currently `1`.
- `device_id`, `pc_id`: the random 128-bit id as 32 lowercase hex characters.
- `caps` (phone): list of what the phone can send right now. `["audio"]` in Phase 1, `["audio", "video"]` from Phase 3.
- Receivers ignore JSON fields and frame types they don't know, so either side can add new ones without breaking the other.

## Media header (inside AUDIO/VIDEO payloads)

```
┌─────────────┬──────────────────┬───────────┬──────────────┐
│ seq (u32)   │ capture_ts (u64, │ codec (u8)│ reserved (u8)│ + data
│             │ µs, phone clock) │           │              │
└─────────────┴──────────────────┴───────────┴──────────────┘
codec: 0x01 PCM s16le 48k mono · 0x02 Opus · 0x10 JPEG
```

`seq` gives loss/reorder stats; `capture_ts` lets the PC measure latency trends, drive the drift resampler, and align video with audio.

## Handshake

```
phone                                PC
  │── HELLO(level, token?) ─────────►│  version ok? token valid? trust rules ([sessions-trust.md](sessions-trust.md))
  │◄──────────── WELCOME ────────────│  (or PENDING … then WELCOME / REJECT)
  │── CONTROL(current settings) ────►│
  │── AUDIO/VIDEO/HEARTBEAT … ──────►│
```

Version rule: `proto` major mismatch → `REJECT(version)` and both sides tell the user to update the older one.

## Liveness

- Heartbeat every **2 s** in both directions when no other frame was sent in that window.
- No frame received for **6 s** → transport considered dead → handover/reconnect logic.
- Socket errors (e.g. cable pulled) trigger this immediately, without waiting.
- Reconnect backoff: 0.5 s → 1 s → 2 s → 4 s → 5 s cap; resets on success.

## Future: UDP media on Wi‑Fi (Phase 5, optional)

If testing shows TCP head-of-line blocking causes audible stalls on busy Wi‑Fi, audio moves to UDP on Level 4 (same media header, Opus in-band FEC + loss concealment), while HELLO/CONTROL/HEARTBEAT stay on TCP. Not built unless measured to be necessary.
