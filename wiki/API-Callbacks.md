# API: Callbacks

`client.callback` lets you subscribe to Steam's asynchronous notifications — a friend changed state, a lobby member left, a peer wants to open a P2P session, the player clicked "Join game" in the overlay.

```ts
const steamworks = require('steamworks.js')
const client = steamworks.init(480)

const handle = client.callback.register(
    steamworks.SteamCallback.PersonaStateChange,
    ({ steam_id, flags }) => {
        console.log('persona changed', steam_id, flags.bits)
    }
)

// Later
handle.disconnect()
```

Callbacks fire from the `runCallbacks` pump that `steamworks.init()` starts at 30 Hz. **If you block the event loop, no callback runs** — and neither do the promises elsewhere in the library.

---

## `register`

```ts
function register<C extends keyof CallbackReturns>(
    steamCallback: C,
    handler: (value: CallbackReturns[C]) => void
): Handle
```

Registers a handler for one callback id and returns a `Handle`. The handler's parameter type is inferred from the id, so `steamCallback` and `handler` stay in sync without any casting.

The enum is reachable two ways — they are the same object:

```ts
steamworks.SteamCallback.PersonaStateChange     // re-exported from the package root
client.callback.SteamCallback.PersonaStateChange
```

You may register several handlers for the same id; each gets its own `Handle`.

## `Handle`

```ts
class Handle {
    disconnect(): void
}
```

`disconnect()` unregisters the handler. Calling it twice is harmless.

> **Keep the handle.** Registrations live in a process-global registry, so a `Handle` you drop on the floor is not garbage collected away — the handler stays subscribed for the lifetime of the process with no way to reach it. Store handles you may want to cancel, and disconnect them when the screen or session that owns them goes away.

There is no "unregister everything" call; disconnect the handles you hold.

---

## Callback ids

```ts
const enum SteamCallback {
    PersonaStateChange = 0,
    SteamServersConnected = 1,
    SteamServersDisconnected = 2,
    SteamServerConnectFailure = 3,
    LobbyDataUpdate = 4,
    LobbyChatUpdate = 5,
    P2PSessionRequest = 6,
    P2PSessionConnectFail = 7,
    GameLobbyJoinRequested = 8,
    MicroTxnAuthorizationResponse = 9
}
```

---

## Payloads

> **Payload fields are `snake_case`.** Unlike the rest of the library, these objects are serialized straight out of the underlying Rust structs, so it is `steam_id`, not `steamId`. The declared types in `callbacks.d.ts` are the authority.

### `PersonaStateChange`

```ts
{
    steam_id: bigint
    flags: { bits: number }
}
```

A user's persona data changed — name, online state, avatar, what they are playing. This is the completion signal for `friends.requestUserInformation`.

`flags.bits` is a raw [`EPersonaChange`](https://partner.steamgames.com/doc/api/ISteamFriends#EPersonaChange) bitmask telling you *what* changed. Those constants are not bound by this library; if you need to distinguish an avatar change from a name change, test the bits against Valve's documented values yourself. Most games just re-read whatever they display.

Fires for anyone the client is tracking — friends, lobby members, users you asked about — so filter by `steam_id` if you only care about some of them.

### `SteamServersConnected`

```ts
{}
```

The client (re)connected to the Steam backend. No payload.

### `SteamServersDisconnected`

```ts
{ reason: number }
```

The client lost its connection to Steam. `reason` is an `EResult` value. Achievements, stats, leaderboards and matchmaking will fail until `SteamServersConnected` fires again.

### `SteamServerConnectFailure`

```ts
{
    reason: number
    still_retrying: boolean
}
```

A connection attempt failed. `still_retrying` is `false` when Steam has given up — that is the point to show the player an offline mode.

### `LobbyDataUpdate`

```ts
{
    lobby: bigint
    member: bigint
    success: boolean
}
```

Lobby metadata changed. When `member` equals `lobby`, the **lobby's own** data changed; otherwise it is that member's per-member data. Re-read with `Lobby.getFullData()` — see [[API-Matchmaking]].

### `LobbyChatUpdate`

```ts
{
    lobby: bigint
    user_changed: bigint
    making_change: bigint
    member_state_change: ChatMemberStateChange
}
```

Someone entered or left a lobby. `user_changed` is the affected user, `making_change` is who caused it (the same id for a voluntary join or leave, the kicker for a kick).

```ts
const enum ChatMemberStateChange {
    /** This user has joined or is joining the lobby. */
    Entered = 0,
    /** This user has left or is leaving the lobby. */
    Left = 1,
    /** User disconnected without leaving the lobby first. */
    Disconnected = 2,
    /** The user has been kicked. */
    Kicked = 3,
    /** The user has been kicked and banned. */
    Banned = 4
}
```

This enum is declared in `callbacks.d.ts` for typing only — there is no runtime object behind it, so compare against the numeric values (or import it as a type and let TypeScript inline the constants, which requires `isolatedModules` to be off).

### `P2PSessionRequest`

```ts
{ remote: bigint }
```

A peer wants to open a P2P session. Nothing they send is delivered until you call `client.networking.acceptP2PSession(remote)` — see [[API-Networking]]. Ignoring the event is how you refuse.

### `P2PSessionConnectFail`

```ts
{
    remote: bigint
    error: number
}
```

A P2P session with `remote` failed. `error` is an `EP2PSessionError` value.

### `GameLobbyJoinRequested`

```ts
{
    lobby_steam_id: bigint
    friend_steam_id: bigint
}
```

The player clicked "Join game" on a friend in the Steam UI while your game was **already running**. Pass `lobby_steam_id` to `client.matchmaking.joinLobby()`.

If the game was *not* running, Steam launches it with the `connect` rich presence command line instead of firing this callback — handle both paths. See `setRichPresence` on [[API-Apps-Utils-and-LocalPlayer]].

### `MicroTxnAuthorizationResponse`

```ts
{
    app_id: number
    order_id: number | bigint
    authorized: boolean
}
```

The player accepted or declined an in-game purchase prompt. `authorized` is the answer; `order_id` is your transaction id, typed `number | bigint` because it is a 64 bit value that may arrive either way — normalize with `BigInt(order_id)` before comparing.

The purchase itself is finalized through the Steam Web API (`ISteamMicroTxn`), not through this library — this callback only tells you the player said yes.

---

## Examples

### Reacting to persona changes

Keeping a friends panel fresh, and using the callback as the "avatar is ready now" signal:

```ts
const steamworks = require('steamworks.js')
const client = steamworks.init(480)

const tracked = new Set<bigint>()

for (const friend of client.friends.getFriends()) {
    const id = friend.getSteamId().steamId64
    tracked.add(id)
    // false → also fetch the avatar
    client.friends.requestUserInformation(id, false)
}

const personaHandle = client.callback.register(
    steamworks.SteamCallback.PersonaStateChange,
    ({ steam_id }) => {
        if (!tracked.has(steam_id)) return

        const friend = client.friends.getFriend(steam_id)
        const avatar = friend.mediumAvatar()   // now likely non-null

        updateRow({
            steamId64: steam_id,
            name: friend.getNickName() ?? friend.getName(),
            state: friend.getState(),
            avatarBytes: avatar?.length ?? 0
        })
    }
)

// When the panel closes
personaHandle.disconnect()
```

### Lobby session plumbing

```ts
const handles = [
    client.callback.register(
        steamworks.SteamCallback.GameLobbyJoinRequested,
        async ({ lobby_steam_id, friend_steam_id }) => {
            console.log('joining', friend_steam_id, 'in', lobby_steam_id)
            const lobby = await client.matchmaking.joinLobby(lobby_steam_id)
            enterMatch(lobby)
        }
    ),

    client.callback.register(
        steamworks.SteamCallback.LobbyChatUpdate,
        ({ lobby, user_changed, member_state_change }) => {
            // 0 Entered, 1 Left, 2 Disconnected, 3 Kicked, 4 Banned
            console.log(`lobby ${lobby}: ${user_changed} -> state ${member_state_change}`)
        }
    ),

    client.callback.register(
        steamworks.SteamCallback.LobbyDataUpdate,
        ({ lobby, member, success }) => {
            if (success && lobby === member) refreshLobbySettings(lobby)
        }
    ),

    client.callback.register(
        steamworks.SteamCallback.SteamServersDisconnected,
        ({ reason }) => showOfflineBanner(reason)
    ),

    client.callback.register(
        steamworks.SteamCallback.SteamServersConnected,
        () => hideOfflineBanner()
    )
]

function teardown() {
    for (const handle of handles) handle.disconnect()
    handles.length = 0
}
```

### Gotchas

- **Handlers must not throw.** An exception escaping a handler surfaces on the callback pump, not at a call site you control — wrap risky work in a `try`.
- **Handlers run on the JS main thread**, invoked from the `runCallbacks` interval. Keep them short; do slow work on a later tick.
- **`snake_case` payloads.** Easy to miss when the rest of the library is camelCase.
- **Not every Steam callback is bound.** The ten ids above are the whole surface — there is no overlay-activated (`GameOverlayActivated_t`), item-installed, or achievement-stored callback in the current bindings.

---

## See also

- [[API-Friends]] — what `PersonaStateChange` tells you to re-read
- [[API-Matchmaking]] — `LobbyDataUpdate`, `LobbyChatUpdate`, `GameLobbyJoinRequested`
- [[API-Networking]] — `P2PSessionRequest`, `P2PSessionConnectFail`
- [[API-Apps-Utils-and-LocalPlayer]] — the `connect` rich presence key that pairs with `GameLobbyJoinRequested`
