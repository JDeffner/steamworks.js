# API: Friends

`client.friends` binds [ISteamFriends](https://partner.steamgames.com/doc/api/ISteamFriends): the friends list, other users' persona names and states, what they are playing, and their avatars.

```ts
const steamworks = require('steamworks.js')
const client = steamworks.init(480)

for (const friend of client.friends.getFriends()) {
    console.log(friend.getName(), friend.getState())
}
```

Every call is synchronous and reads whatever the Steam client currently has cached. For users who are not friends, that cache may be empty until you ask for it — see [`requestUserInformation`](#requestuserinformation).

> Rich presence for the **local** player lives on `client.localplayer.setRichPresence` — see [[API-Apps-Utils-and-LocalPlayer]].

---

## Listing users

### `getFriends`

```ts
function getFriends(flags?: number | null): Array<Friend>
```

Returns the users matching the given relationship. Omitting `flags` gives you the regular friends list (`FriendFlags.Immediate`).

[`GetFriendByIndex`](https://partner.steamgames.com/doc/api/ISteamFriends#GetFriendByIndex)

```ts
const friends = client.friends.getFriends()
const clanmates = client.friends.getFriends(client.friends.FriendFlags.ClanMember)
```

### `FriendFlags`

```ts
const enum FriendFlags {
    None = 0,
    Blocked = 1,
    FriendshipRequested = 2,
    /** The usual friends list. */
    Immediate = 4,
    ClanMember = 8,
    OnGameServer = 16,
    RequestingFriendship = 128,
    RequestingInfo = 256,
    Ignored = 512,
    IgnoredFriend = 1024,
    ChatMember = 4096,
    All = 65535
}
```

`flags` is a **bitmask** — the values may be OR-ed together, which is why the parameter is typed `number` rather than the enum itself.

```ts
// Everyone the local user has blocked or ignored
const blockedOrIgnored = client.friends.getFriends(
    client.friends.FriendFlags.Blocked | client.friends.FriendFlags.Ignored
)
```

Bits that are not part of `EFriendFlags` are silently dropped rather than rejected, so a stray bit will not throw — it just will not match anything.

[`EFriendFlags`](https://partner.steamgames.com/doc/api/ISteamFriends#EFriendFlags)

### `getCoplayFriends`

```ts
function getCoplayFriends(): Array<Friend>
```

The local user's recently-played-with list — people they were in a game session with, whether or not they are friends. Useful for an "add the people you just played with" prompt.

[`GetCoplayFriend`](https://partner.steamgames.com/doc/api/ISteamFriends#GetCoplayFriend)

### `getFriend`

```ts
function getFriend(steamId64: bigint): Friend
```

Wraps an arbitrary steam id in a `Friend` handle. **They do not have to be a friend** — this is how you look up a lobby member, an opponent, or the sender of a P2P packet.

```ts
const other = client.friends.getFriend(76561197960287930n)
console.log(other.getName())
```

This call always succeeds and never validates the id; the getters on the returned handle are what tell you whether Steam knows anything about that user. For a stranger, expect `getName()` to be empty and the avatars to be `null` until you call `requestUserInformation`.

### `requestUserInformation`

```ts
function requestUserInformation(steamId64: bigint, nameOnly: boolean): boolean
```

Asks Steam to download the persona name — and, when `nameOnly` is `false`, the avatar — of a user.

Returns `true` if the information is *being fetched*, `false` if it was already available. So `false` is the good case: the data is ready right now.

[`RequestUserInformation`](https://partner.steamgames.com/doc/api/ISteamFriends#RequestUserInformation)

```ts
const steamId = 76561197960287930n
if (client.friends.requestUserInformation(steamId, false)) {
    // Not cached yet: wait for the PersonaStateChange callback before reading
} else {
    const friend = client.friends.getFriend(steamId)
    console.log(friend.getName(), friend.mediumAvatar()?.length)
}
```

The completion signal is the `PersonaStateChange` callback — see [[API-Callbacks]].

---

## `Friend`

```ts
class Friend {
    getSteamId(): PlayerSteamId
    getName(): string
    getNickName(): string | null
    getState(): FriendState
    getGamePlayed(): FriendGame | null
    hasFriend(flags: number): boolean
    smallAvatar(): Buffer | null
    mediumAvatar(): Buffer | null
    largeAvatar(): Buffer | null
}
```

A `Friend` is a handle around a steam id; it caches nothing, so every getter reflects the Steam client's current view.

### Identity

```ts
const id = friend.getSteamId()
// { steamId64: 76561197960287930n, steamId32: 'STEAM_0:0:11101', accountId: 22202 }
```

```ts
friend.getName()      // the current persona name, e.g. 'gaben'
friend.getNickName()  // the nickname the local user set for them, or null
```

Show `getNickName() ?? getName()` if you want to respect the player's own labelling.

### `getState`

```ts
getState(): FriendState
```

```ts
const enum FriendState {
    Offline = 0,
    Online = 1,
    Busy = 2,
    Away = 3,
    Snooze = 4,
    LookingToTrade = 5,
    LookingToPlay = 6,
    Invisible = 7
}
```

`Invisible` is handled explicitly by this fork — a user in invisible mode used to abort the process through the underlying crate. Any state a future SDK adds is reported as `Offline` rather than throwing.

[`EPersonaState`](https://partner.steamgames.com/doc/api/ISteamFriends#EPersonaState)

### `getGamePlayed`

```ts
getGamePlayed(): FriendGame | null
```

`null` when the user is not in a game.

```ts
interface FriendGame {
    /** The id of the game being played. */
    gameId: bigint
    /** The app id of the game being played. */
    appId: number
    /** The IPv4 address of the server the player is on, "0.0.0.0" if none. */
    gameAddress: string
    /** The game port of the server the player is on, 0 if none. */
    gamePort: number
    /** The query port of the server the player is on, 0 if none. */
    queryPort: number
    /** The id of the lobby the player is in, 0 if none. */
    lobbyId: bigint
}
```

`lobbyId` is the hook for "join my friend's game": when it is non-zero and `appId` matches yours, pass it to `client.matchmaking.joinLobby(...)` — see [[API-Matchmaking]].

```ts
const game = friend.getGamePlayed()
if (game !== null && game.appId === client.utils.getAppId() && game.lobbyId !== 0n) {
    const lobby = await client.matchmaking.joinLobby(game.lobbyId)
    console.log(`joined ${friend.getName()}'s lobby`, lobby.id)
}
```

[`GetFriendGamePlayed`](https://partner.steamgames.com/doc/api/ISteamFriends#GetFriendGamePlayed)

### `hasFriend`

```ts
hasFriend(flags: number): boolean
```

Whether this user matches the given relationship criteria. Same bitmask as `getFriends`.

```ts
friend.hasFriend(client.friends.FriendFlags.OnGameServer)
friend.hasFriend(
    client.friends.FriendFlags.Immediate | client.friends.FriendFlags.ClanMember
)
```

---

## Avatars

```ts
smallAvatar(): Buffer | null    //  32 x  32 RGBA =   4096 bytes
mediumAvatar(): Buffer | null   //  64 x  64 RGBA =  16384 bytes
largeAvatar(): Buffer | null    // 184 x 184 RGBA = 135424 bytes
```

Each returns **raw RGBA bytes** — not a PNG, not a data URL. Four bytes per pixel, row-major, top-left origin.

`null` means the avatar is not in the Steam client's cache yet. Call `requestUserInformation(steamId, false)` and try again after the `PersonaStateChange` callback fires.

### Rendering in a browser or Electron renderer

An RGBA buffer maps directly onto `ImageData`, so a canvas can draw it with no decoding step:

```ts
function avatarToDataUrl(rgba: Buffer, size: number): string {
    const canvas = document.createElement('canvas')
    canvas.width = size
    canvas.height = size

    const ctx = canvas.getContext('2d')!
    // Copy into a fresh Uint8ClampedArray; ImageData will not take a Node Buffer view directly
    const image = new ImageData(new Uint8ClampedArray(rgba), size, size)
    ctx.putImageData(image, 0, 0)

    return canvas.toDataURL('image/png')
}

const avatar = friend.mediumAvatar()
if (avatar !== null) {
    img.src = avatarToDataUrl(avatar, 64)
}
```

### Encoding to a PNG in the main process

With any raw-pixel encoder (`sharp`, `jimp`, `pngjs`, …) — `sharp` shown here:

```ts
import sharp from 'sharp'

const avatar = friend.largeAvatar()
if (avatar !== null) {
    await sharp(avatar, { raw: { width: 184, height: 184, channels: 4 } })
        .png()
        .toFile('avatar.png')
}
```

The buffer length is a reliable sanity check: `avatar.length === size * size * 4`.

---

## Complete example

```ts
const steamworks = require('steamworks.js')
const client = steamworks.init(480)

// Ask Steam for anything missing before we render
for (const friend of client.friends.getFriends()) {
    client.friends.requestUserInformation(friend.getSteamId().steamId64, false)
}

client.callback.register(steamworks.SteamCallback.PersonaStateChange, () => {
    render()
})

function render() {
    for (const friend of client.friends.getFriends(client.friends.FriendFlags.Immediate)) {
        const id = friend.getSteamId()
        const game = friend.getGamePlayed()
        const avatar = friend.smallAvatar()

        console.log({
            steamId64: id.steamId64,
            name: friend.getNickName() ?? friend.getName(),
            state: friend.getState(),
            playing: game?.appId ?? null,
            avatarBytes: avatar?.length ?? 0
        })
    }
}

render()
```

---

## See also

- [[API-Callbacks]] — `PersonaStateChange` tells you when names, states and avatars have changed
- [[API-Overlay]] — `activateDialogToUser` to open a chat or profile for a `Friend`
- [[API-Matchmaking]] — joining the lobby from `getGamePlayed().lobbyId`
- [[API-Apps-Utils-and-LocalPlayer]] — the local user's own id, name and rich presence
