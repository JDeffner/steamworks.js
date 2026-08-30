# API: Matchmaking

`client.matchmaking` binds [ISteamMatchmaking](https://partner.steamgames.com/doc/api/ISteamMatchmaking): creating and joining lobbies, reading and writing lobby metadata, and listing the lobbies of your app with server-side filters.

```ts
const steamworks = require('steamworks.js')
const client = steamworks.init(480)

const lobby = await client.matchmaking.createLobby(
    client.matchmaking.LobbyType.Public,
    8
)
```

All lobby ids are `bigint` — they are 64 bit values and lose precision as JS `number`. So are `getMemberCount()` and `getMemberLimit()`, which return `bigint` even though they are small counts.

> **Steam must be running.** Every call here goes through the initialized Steam client; calling into the API before `steamworks.init()` succeeded will crash the process rather than throw. Promises resolve from Steam callbacks, which are pumped by the 30 Hz interval `init()` starts for you — don't block the event loop while awaiting them.

---

## Lobby lifecycle

### `createLobby`

```ts
function createLobby(lobbyType: LobbyType, maxMembers: number): Promise<Lobby>
```

Creates a lobby owned by the local player and resolves with it once Steam confirms creation. Rejects with Steam's error text on failure.

`maxMembers` is the member limit including the owner.

[`CreateLobby`](https://partner.steamgames.com/doc/api/ISteamMatchmaking#CreateLobby)

### `LobbyType`

```ts
const enum LobbyType {
    Private = 0,
    FriendsOnly = 1,
    Public = 2,
    Invisible = 3
}
```

| Value | Who can find it |
| --- | --- |
| `Private` | Invite only; never returned by `getLobbies` |
| `FriendsOnly` | Visible to the owner's friends, invite still works |
| `Public` | Listed by `getLobbies` for everyone |
| `Invisible` | Joinable, but the user does not appear to be in it in the friends UI |

### `joinLobby`

```ts
function joinLobby(lobbyId: bigint): Promise<Lobby>
```

Joins a lobby by id. Rejects with `Failed to join lobby` on any failure — Steam's specific reason (full, banned, does not exist) is **not** surfaced, so treat a rejection as a generic "could not join".

```ts
const lobby = await client.matchmaking.joinLobby(109775240990226795n)
```

### `Lobby`

```ts
class Lobby {
    id: bigint
    join(): Promise<Lobby>
    leave(): void
    openInviteDialog(): void
    getMemberCount(): bigint
    getMemberLimit(): bigint | null
    getMembers(): Array<PlayerSteamId>
    getOwner(): PlayerSteamId
    setJoinable(joinable: boolean): boolean
    getData(key: string): string | null
    setData(key: string, value: string): boolean
    deleteData(key: string): boolean
    getFullData(): Record<string, string>
    mergeFullData(data: Record<string, string>): boolean
}
```

A `Lobby` is a thin handle around a lobby id — it holds no cached state, every method asks Steam. Instances come back from `createLobby`, `joinLobby` and `getLobbies`; a `Lobby` from `getLobbies` is a lobby you have *not* joined yet, so call `join()` on it first.

#### Membership

```ts
lobby.getMemberCount()          // 3n
lobby.getMemberLimit()          // 8n, or null when Steam does not know the limit
lobby.getMembers()              // PlayerSteamId[]
lobby.getOwner()                // PlayerSteamId
```

`getMemberLimit()` returns `null` when the limit is unknown to the client — most commonly for a lobby you have not joined and whose data has not arrived yet.

Every `PlayerSteamId` is:

```ts
interface PlayerSteamId {
    steamId64: bigint
    steamId32: string
    accountId: number
}
```

Member data only stays fresh while callbacks run. Subscribe to `LobbyChatUpdate` to be told when members join or leave — see [[API-Callbacks]].

#### Joinability and invites

```ts
lobby.setJoinable(true)         // false when the local user is not the owner
lobby.openInviteDialog()        // Steam overlay invite dialog for this lobby
lobby.leave()
```

`openInviteDialog()` is the same call as `overlay.activateInviteDialog(lobby.id)` — see [[API-Overlay]]. It needs the Steam overlay to be available (in Electron, see [[Installation]]).

`leave()` returns nothing and cannot fail; the handle is dead afterwards.

---

## Lobby data

Lobby metadata is a string→string map replicated to everyone who can see the lobby. It is also what the lobby list filters match against, so this is how you advertise a game mode, map, or skill bracket.

```ts
lobby.setData('gamemode', 'ffa')      // true when the write was accepted
lobby.getData('gamemode')             // 'ffa' | null
lobby.deleteData('gamemode')          // true when the key was removed
```

Only the lobby owner may write lobby data; `setData`/`deleteData` return `false` otherwise.

### `getFullData` / `mergeFullData`

```ts
getFullData(): Record<string, string>
mergeFullData(data: Record<string, string>): boolean
```

`getFullData()` walks every key Steam has for the lobby and returns them as a plain object.

`mergeFullData()` writes each entry with `setData` and returns `true` only if *all* writes succeeded. It **merges** — keys not present in `data` are left alone, it is not a replace. There is no rollback: a partial failure leaves the earlier keys written.

```ts
lobby.mergeFullData({
    gamemode: 'ffa',
    map: 'crossfire',
    elo: '1500'
})

const all = lobby.getFullData()
console.log(all.map) // 'crossfire'
```

Numbers have to be stringified going in; the numerical filters below parse them back out on Steam's side.

---

## Listing lobbies

### `getLobbies`

```ts
function getLobbies(filter?: LobbyListFilter | null): Promise<Array<Lobby>>
```

Requests the lobby list for this app. Called with no argument it behaves exactly like it always has — the unfiltered list, capped by Steam's own defaults.

[`RequestLobbyList`](https://partner.steamgames.com/doc/api/ISteamMatchmaking#RequestLobbyList)

```ts
// Bare: every listable lobby for this app
const lobbies = await client.matchmaking.getLobbies()
for (const lobby of lobbies) {
    console.log(lobby.id, lobby.getMemberCount(), lobby.getData('gamemode'))
}
```

### `LobbyListFilter`

New in 0.6. Every field is optional; an empty object returns the same lobbies as an unfiltered request.

```ts
interface LobbyListFilter {
    /** String metadata comparisons a lobby has to satisfy */
    stringFilters?: Array<LobbyStringFilter>
    /** Numerical metadata comparisons a lobby has to satisfy */
    numberFilters?: Array<LobbyNumberFilter>
    /** Metadata values the results are sorted closest to */
    nearValueFilters?: Array<LobbyNearFilter>
    /** Only return lobbies with at least this many open slots (0-255) */
    slotsAvailable?: number
    /** How far geographically the returned lobbies may be */
    distance?: LobbyDistanceFilter
    /** Maximum amount of lobbies to return */
    count?: number
}
```

Filters are evaluated by Steam before the list comes back, so they cost you nothing client-side — prefer them over fetching everything and filtering in JS.

#### String comparisons

```ts
interface LobbyStringFilter {
    key: string
    value: string
    comparison: LobbyStringComparison
}

const enum LobbyStringComparison {
    EqualToOrLessThan = 0,
    LessThan = 1,
    Equal = 2,
    GreaterThan = 3,
    EqualToOrGreaterThan = 4,
    NotEqual = 5
}
```

Matches lobbies whose string metadata under `key` compares against `value` as requested. `Equal` and `NotEqual` are the useful ones for tags like a game mode; the ordering comparisons apply Steam's own string ordering.

[`AddRequestLobbyListStringFilter`](https://partner.steamgames.com/doc/api/ISteamMatchmaking#AddRequestLobbyListStringFilter)

#### Numerical comparisons

```ts
interface LobbyNumberFilter {
    key: string
    value: number
    comparison: LobbyNumberComparison
}

const enum LobbyNumberComparison {
    Equal = 0,
    NotEqual = 1,
    GreaterThan = 2,
    GreaterThanEqualTo = 3,
    LessThan = 4,
    LessThanEqualTo = 5
}
```

`value` is a signed 32 bit integer on the Rust side — pass whole numbers within `±2147483647`. The lobby's metadata value under `key` is parsed as a number by Steam, so the owner has to have written something numeric (`lobby.setData('elo', '1500')`).

Note that the two comparison enums are **not** interchangeable: `LobbyStringComparison.Equal` is `2` while `LobbyNumberComparison.Equal` is `0`. Always use the enum that matches the filter type.

[`AddRequestLobbyListNumericalFilter`](https://partner.steamgames.com/doc/api/ISteamMatchmaking#AddRequestLobbyListNumericalFilter)

#### Near-value sorting

```ts
interface LobbyNearFilter {
    key: string
    value: number
}
```

Sorts the results by how close their metadata under `key` is to `value`. This **does not filter anything out** — it only orders the results, so combine it with a numerical filter if you also want a hard bound.

[`AddRequestLobbyListNearValueFilter`](https://partner.steamgames.com/doc/api/ISteamMatchmaking#AddRequestLobbyListNearValueFilter)

#### Slots, distance and count

```ts
slotsAvailable?: number  // 0-255, lobbies with at least this many free seats
count?: number           // maximum lobbies to return
distance?: LobbyDistanceFilter
```

```ts
const enum LobbyDistanceFilter {
    Close = 0,       // same region only
    Default = 1,     // same region, or nearby regions
    Far = 2,         // includes distant regions
    Worldwide = 3    // no geographic limit
}
```

`slotsAvailable` is a `u8` — values above 255 will fail to convert. Omitting `distance` leaves Steam's default (`Default`).

### Worked example

```ts
import type { matchmaking } from 'steamworks.js/client'

const filter: matchmaking.LobbyListFilter = {
    stringFilters: [{
        key: 'gamemode',
        value: 'ffa',
        comparison: client.matchmaking.LobbyStringComparison.Equal
    }],
    numberFilters: [{
        key: 'elo',
        value: 1500,
        comparison: client.matchmaking.LobbyNumberComparison.GreaterThanEqualTo
    }],
    nearValueFilters: [{ key: 'elo', value: 1800 }],
    slotsAvailable: 2,
    distance: client.matchmaking.LobbyDistanceFilter.Far,
    count: 20
}

const lobbies = await client.matchmaking.getLobbies(filter)
const best = lobbies[0]           // closest to elo 1800, thanks to the near filter
if (best !== undefined) {
    const joined = await best.join()
    console.log(joined.getData('gamemode'), joined.getMemberCount())
    joined.leave()
}
```

And the owner side that makes those filters match:

```ts
const lobby = await client.matchmaking.createLobby(client.matchmaking.LobbyType.Public, 8)
lobby.mergeFullData({ gamemode: 'ffa', elo: '1750', map: 'crossfire' })
lobby.setJoinable(true)
```

### Gotchas

- **Filter keys are validated.** A key containing a null byte, or one Steam refuses (it takes keys as C strings of at most 255 bytes), rejects the promise with a descriptive error instead of crashing. Values are checked for null bytes too.
- **Concurrent calls are serialized.** Steam keeps pending lobby-list filters in process-global state, so the binding takes an internal lock across applying filters and issuing the request. Two overlapping `getLobbies` calls will not mix each other's filters, but they do queue briefly.
- **Filters are one-shot.** They apply to that single request only; the next `getLobbies()` starts clean.
- **`Private` lobbies are never listed** regardless of filters.

---

## See also

- [[API-Callbacks]] — `LobbyDataUpdate`, `LobbyChatUpdate`, `GameLobbyJoinRequested` for reacting to lobby changes and overlay "Join game" clicks
- [[API-Overlay]] — the invite dialog
- [[API-Networking]] — P2P messaging between lobby members
- [[API-Friends]] — `getGamePlayed().lobbyId` to find the lobby a friend is in
