# API: Networking

`client.networking` binds the peer-to-peer packet functions of [ISteamNetworking](https://partner.steamgames.com/doc/api/ISteamNetworking): send a buffer to another steam id, poll for incoming buffers, and accept incoming sessions.

```ts
const steamworks = require('steamworks.js')
const client = steamworks.init(480)

// Accept anyone who wants to talk to us
client.callback.register(steamworks.SteamCallback.P2PSessionRequest, ({ remote }) => {
    client.networking.acceptP2PSession(remote)
})

client.networking.sendP2PPacket(
    76561197960287930n,
    client.networking.SendType.Reliable,
    Buffer.from('hello')
)
```

Steam relays or hole-punches for you — you address peers by steam id, never by IP. Steam ids are `bigint`.

---

## `sendP2PPacket`

```ts
function sendP2PPacket(steamId64: bigint, sendType: SendType, data: Buffer): boolean
```

Sends a packet to a peer. Returns `true` when Steam accepted the packet for sending — that is *not* delivery confirmation, only that the send was queued.

Sending to a peer with no session yet implicitly opens one: the remote side gets a `P2PSessionRequest` callback and must call `acceptP2PSession` before anything is delivered. Packets sent before acceptance are buffered (except with `UnreliableNoDelay`).

[`SendP2PPacket`](https://partner.steamgames.com/doc/api/ISteamNetworking#SendP2PPacket)

### `SendType`

```ts
const enum SendType {
    Unreliable = 0,
    UnreliableNoDelay = 1,
    Reliable = 2,
    ReliableWithBuffering = 3
}
```

| Value | Behavior |
| --- | --- |
| `Unreliable` | Sends the packet directly over UDP. **Can't be larger than 1200 bytes.** |
| `UnreliableNoDelay` | Like `Unreliable`, but doesn't buffer packets sent before the connection has started — they are dropped instead. |
| `Reliable` | Reliable packet sending. **Can't be larger than 1 megabyte.** |
| `ReliableWithBuffering` | Like `Reliable`, but applies the Nagle algorithm to packets being sent — fewer, larger datagrams at the cost of a little latency. |

Use `Unreliable` for per-frame state you will resend anyway (positions, inputs), `Reliable` for things that must arrive (chat, spawn events, an auth ticket).

---

## `isP2PPacketAvailable`

```ts
function isP2PPacketAvailable(): number
```

The size in bytes of the next packet waiting to be read, or `0` when nothing is queued. This is the value to pass to `readP2PPacket`.

[`IsP2PPacketAvailable`](https://partner.steamgames.com/doc/api/ISteamNetworking#IsP2PPacketAvailable)

---

## `readP2PPacket`

```ts
function readP2PPacket(size: number): P2PPacket

interface P2PPacket {
    data: Buffer
    size: number
    steamId: PlayerSteamId
}
```

Reads the next queued packet into a buffer of `size` bytes.

**Throws** `No packet available` when the queue is empty — always gate it on `isP2PPacketAvailable()` rather than calling speculatively, or wrap it in a `try`.

> **`data` is the full allocated buffer, not the packet.** It is always exactly `size` bytes long; `size` on the returned object is how many bytes Steam actually wrote. When you pass a `size` larger than the packet, the tail is zero padding. Slice it:
>
> ```ts
> const payload = packet.data.subarray(0, packet.size)
> ```

`steamId` is the sender:

```ts
interface PlayerSteamId {
    steamId64: bigint
    steamId32: string
    accountId: number
}
```

[`ReadP2PPacket`](https://partner.steamgames.com/doc/api/ISteamNetworking#ReadP2PPacket)

### Draining the queue

Packets arrive faster than one per tick — read until the queue is empty:

```ts
function pump() {
    let size: number
    while ((size = client.networking.isP2PPacketAvailable()) > 0) {
        const packet = client.networking.readP2PPacket(size)
        const payload = packet.data.subarray(0, packet.size)
        onMessage(packet.steamId.steamId64, payload)
    }
}

setInterval(pump, 1000 / 30)
```

Nothing in the library polls for you — incoming packets sit in Steam's queue until you read them. Pick a cadence that matches your tick rate.

---

## `acceptP2PSession`

```ts
function acceptP2PSession(steamId64: bigint): void
```

Accepts an incoming P2P session from a peer. Until you do, packets that peer sends are not delivered to you.

[`AcceptP2PSessionWithUser`](https://partner.steamgames.com/doc/api/ISteamNetworking#AcceptP2PSessionWithUser)

The trigger is the `P2PSessionRequest` callback — see [[API-Callbacks]]:

```ts
client.callback.register(steamworks.SteamCallback.P2PSessionRequest, ({ remote }) => {
    if (isExpectedPeer(remote)) {
        client.networking.acceptP2PSession(remote)
    }
    // Ignoring the request is how you refuse; there is no explicit reject
})
```

**Only accept peers you expect** — someone in your lobby, someone on your server's roster. Accepting every request exposes your IP to whoever asks. There is no `closeP2PSession` binding in the current surface, so a session stays open until Steam times it out.

---

## Complete example

Two peers in a lobby exchanging JSON messages:

```ts
const steamworks = require('steamworks.js')
const client = steamworks.init(480)

const lobby = await client.matchmaking.joinLobby(109775240990226795n)
const expected = new Set(lobby.getMembers().map(m => m.steamId64))

client.callback.register(steamworks.SteamCallback.P2PSessionRequest, ({ remote }) => {
    if (expected.has(remote)) {
        client.networking.acceptP2PSession(remote)
    } else {
        console.warn('ignoring P2P request from', remote)
    }
})

client.callback.register(steamworks.SteamCallback.P2PSessionConnectFail, ({ remote, error }) => {
    console.error(`P2P connection to ${remote} failed, error ${error}`)
})

function send(peer: bigint, message: unknown) {
    const ok = client.networking.sendP2PPacket(
        peer,
        client.networking.SendType.Reliable,
        Buffer.from(JSON.stringify(message), 'utf8')
    )
    if (!ok) console.warn('Steam refused the packet for', peer)
}

setInterval(() => {
    let size: number
    while ((size = client.networking.isP2PPacketAvailable()) > 0) {
        const packet = client.networking.readP2PPacket(size)
        const text = packet.data.subarray(0, packet.size).toString('utf8')
        console.log(packet.steamId.steamId64, JSON.parse(text))
    }
}, 1000 / 30)

for (const member of lobby.getMembers()) {
    if (member.steamId64 !== client.localplayer.getSteamId().steamId64) {
        send(member.steamId64, { type: 'hello' })
    }
}
```

### Gotchas

- **Session callbacks need the callback pump.** `steamworks.init()` starts it at 30 Hz; `P2PSessionRequest` will never fire if you block the event loop.
- **Respect the size limits.** Over 1200 bytes on an unreliable send, or over 1 MB reliable, and the send fails.
- **`readP2PPacket` throws when empty** — it is not a `null`-returning API.
- **No `closeP2PSession`.** You cannot explicitly tear a session down through this binding.
- This is Steam's older P2P API. The newer `ISteamNetworkingSockets` / `ISteamNetworkingMessages` interfaces are not bound.

---

## See also

- [[API-Callbacks]] — `P2PSessionRequest`, `P2PSessionConnectFail`
- [[API-Matchmaking]] — lobbies as the place to discover who to connect to
- [[API-Auth]] — proving who a peer is before trusting them
