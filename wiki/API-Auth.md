# API: Auth

`client.auth` binds the session-ticket half of [ISteamUser](https://partner.steamgames.com/doc/api/ISteamUser): the local player asks Steam for a signed ticket proving who they are, and hands it to a peer, a game server, or your web backend to verify.

```ts
const steamworks = require('steamworks.js')
const client = steamworks.init(480)

const ticket = await client.auth.getAuthTicketForWebApi('mygame-backend')
const hex = ticket.getBytes().toString('hex')
// POST hex to your server, which validates it with the Steam Web API
ticket.cancel()
```

Three functions produce a ticket, differing only in **who the ticket is issued to**. All three are `Promise`-based: they wait for Steam to confirm the ticket is valid before resolving, and reject if it does not.

---

## `getAuthTicketForWebApi`

```ts
function getAuthTicketForWebApi(
    identity: string,
    timeoutSeconds?: number | null
): Promise<Ticket>
```

Requests a ticket intended for verification through the **Steam Web API** — this is the right call for a web/HTTP backend.

- `identity` — an arbitrary string naming the service that will verify the ticket. It is baked into the ticket, and your backend must pass the *same* string as `identity` when it validates, or validation fails. Pick one constant for your service and use it on both sides.
- `timeoutSeconds` — how long to wait for Steam to validate before rejecting. **Defaults to 10 seconds.**

[`GetAuthTicketForWebApi`](https://partner.steamgames.com/doc/api/ISteamUser#GetAuthTicketForWebApi)

```ts
const ticket = await client.auth.getAuthTicketForWebApi('mygame-backend')
const res = await fetch('https://api.mygame.example/login', {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify({ ticket: ticket.getBytes().toString('hex') })
})
```

The ticket data resolved here is already truncated to the real ticket length — `getBytes()` gives you exactly the bytes to send, no trailing padding.

---

## `getSessionTicketWithSteamId`

```ts
function getSessionTicketWithSteamId(
    steamId64: bigint,
    timeoutSeconds?: number | null
): Promise<Ticket>
```

Requests a ticket bound to a **remote user or game server identified by steam id**.

- `steamId64` — the user steam id or game server steam id. Use as the network identity of the remote system that will authenticate the ticket. If it is peer-to-peer, the user steam id; if it is a game server, the game server steam id may be used when it was obtained from a trusted third party.
- `timeoutSeconds` — defaults to 10 seconds.

[`GetAuthSessionTicket`](https://partner.steamgames.com/doc/api/ISteamUser#GetAuthSessionTicket)

```ts
// Authenticating to a peer in a P2P session
const peer = 76561197960287930n
const ticket = await client.auth.getSessionTicketWithSteamId(peer)
client.networking.sendP2PPacket(peer, client.networking.SendType.Reliable, ticket.getBytes())
```

---

## `getSessionTicketWithIp`

```ts
function getSessionTicketWithIp(
    ip: string,
    timeoutSeconds?: number | null
): Promise<Ticket>
```

Requests a ticket bound to a remote system identified by **network address** — for a game server whose steam id you do not have.

`ip` is parsed as a **socket address, so it must include the port**:

```ts
await client.auth.getSessionTicketWithIp('203.0.113.10:27015')   // IPv4
await client.auth.getSessionTicketWithIp('[2001:db8::1]:27015')  // IPv6, bracketed
```

A bare address with no port rejects with the parse error (`invalid socket address syntax`) rather than a Steam error. `timeoutSeconds` defaults to 10 seconds.

---

## `Ticket`

```ts
class Ticket {
    getBytes(): Buffer
    cancel(): void
}
```

### `getBytes`

The raw ticket bytes. Send them to whoever is verifying; hex is the conventional encoding for the Web API (`ticket.getBytes().toString('hex')`), while a P2P or game-server path can take the buffer as-is.

Each call returns a fresh copy of the buffer, so mutating it is harmless.

### `cancel`

```ts
ticket.cancel()
```

Tells Steam the ticket is no longer in use — [`CancelAuthTicket`](https://partner.steamgames.com/doc/api/ISteamUser#CancelAuthTicket). The remote side's session for this ticket is dropped, so call it when the player disconnects or logs out, not while the session is still live.

Tickets are a limited resource; **always cancel a ticket you are finished with**. A ticket whose validation failed or timed out is cancelled for you before the promise rejects, so you only need to cancel the ones you successfully received.

`cancel()` is idempotent from JS's point of view — calling it twice will not throw — but there is no `isCancelled` flag, so track it yourself if that matters.

---

## Error conditions

Every ticket function rejects rather than returning a sentinel:

| Situation | Rejection |
| --- | --- |
| Steam reports the ticket invalid | Steam's own result text (e.g. an `EResult` name) |
| No response within `timeoutSeconds` | `Steam didn't validated the ticket in time.` (sic) |
| Malformed `ip` in `getSessionTicketWithIp` | the address parse error |

In every failing case the underlying ticket handle is cancelled before the rejection, so there is nothing to clean up.

```ts
try {
    const ticket = await client.auth.getSessionTicketWithSteamId(peerId, 5)
    // …
} catch (err) {
    console.error('could not get a session ticket:', err)
}
```

> Promises here resolve from Steam callbacks, which are pumped by the 30 Hz interval `steamworks.init()` starts. If you block the event loop while awaiting, the callback never runs and every request times out.

---

## Validating on the server

The ticket only proves anything once the *other side* verifies it with Valve. This library binds the client half; the verification half is not in-process and is not bound here.

### Web backend

Your server posts the hex ticket to [`ISteamUserAuth/AuthenticateUserTicket`](https://partner.steamgames.com/doc/webapi/ISteamUserAuth#AuthenticateUserTicket) with your publisher Web API key, the app id, and the **same `identity` string** you passed to `getAuthTicketForWebApi`. Steam answers with the steam id the ticket belongs to and whether the user owns the app.

```
GET https://api.steampowered.com/ISteamUserAuth/AuthenticateUserTicket/v1/
    ?key=<publisher web api key>
    &appid=480
    &ticket=<hex from getBytes()>
    &identity=mygame-backend
```

Never trust a steam id the client simply tells you — the ticket is the proof. Keep the Web API key server-side.

Ownership follow-up, if you need it: [`ISteamUser/CheckAppOwnership`](https://partner.steamgames.com/doc/webapi/ISteamUser#CheckAppOwnership).

### Game server / peer

A dedicated server or authoritative peer verifies with `ISteamGameServer::BeginAuthSession` (or `ISteamUser::BeginAuthSession` for P2P) and ends the session with `EndAuthSession` when the player leaves.

**Those calls are not bound by this library.** `client.auth` gives you the requesting side only; there is no `beginAuthSession` / `endAuthSession` / `userHasLicenseForApp` in the current surface. A Node game server needs the Web API route above, or its own native binding.

Background: [User Authentication and Ownership](https://partner.steamgames.com/doc/features/auth).

---

## Complete example

```ts
const steamworks = require('steamworks.js')
const client = steamworks.init(480)

let session: { ticket: import('steamworks.js/client').auth.Ticket } | null = null

async function login() {
    const ticket = await client.auth.getAuthTicketForWebApi('mygame-backend', 15)
    session = { ticket }

    const res = await fetch('https://api.mygame.example/session', {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({
            ticket: ticket.getBytes().toString('hex'),
            // The server learns the real steam id from Steam, not from this
            claimedSteamId: client.localplayer.getSteamId().steamId64.toString()
        })
    })

    if (!res.ok) {
        ticket.cancel()
        session = null
        throw new Error('server rejected the ticket')
    }
}

function logout() {
    session?.ticket.cancel()
    session = null
}

process.on('exit', logout)
```

---

## See also

- [[API-Networking]] — sending a ticket to a peer over P2P
- [[API-Apps-Utils-and-LocalPlayer]] — `apps.isSubscribedApp` for a client-side ownership hint (not a substitute for server validation)
- [[API-Callbacks]] — `SteamServersConnected` / `SteamServersDisconnected` for when the client loses its Steam session
