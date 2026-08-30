# Getting Started

This page covers everything that is true of the whole library: initialization, app ids, the callback pump, and the `bigint` rule. Per-namespace detail lives in the API pages linked at the bottom.

## `init(appId?)`

```js
const steamworks = require('steamworks.js')

const client = steamworks.init(480)
```

`init` does three things:

1. Initializes the Steam API. **It throws if that fails** — Steam not running, not logged in, unknown or mismatched app id, missing `steam_appid.txt`. There is no boolean return to check; wrap it in `try`/`catch` if you want to run without Steam.
2. Starts a `setInterval` that calls the native `runCallbacks()` at 30 Hz (every ~33 ms). This is what drives every asynchronous result and every registered callback. You never call `runCallbacks` yourself — it is not on the returned object.
3. Returns the API object: `achievement`, `apps`, `auth`, `callback`, `cloud`, `friends`, `input`, `leaderboard`, `localplayer`, `matchmaking`, `networking`, `overlay`, `stats`, `utils`, `workshop`.

```js
let client
try {
    client = steamworks.init(480)
} catch (error) {
    console.warn('Steam unavailable, running offline:', error.message)
}
```

### Keep the returned object

Everything you call goes through the object `init` returns, so store it somewhere your game can reach — a module-level `const`, your engine's service locator, whatever. There is no `steamworks.getClient()` accessor to fetch it back later.

Call `init` **once**, at startup, before any other Steam call. Calling it again clears the existing callback interval and starts a new one; it is not a way to get a second, independent client. In Electron, initialize in exactly one process (see [[Installation]]).

The 30 Hz interval is a live timer, so a plain Node script will not exit on its own once you have initialized. Call `process.exit()` when your tool is done. There is no `deinit`/`shutdown` binding for the client (`input.shutdown()` shuts down Steam Input only).

### App id: argument vs `steam_appid.txt`

`appId` is optional. If you omit it, Steam looks for a `steam_appid.txt` file — a plain text file containing only the numeric app id — in the current working directory.

```js
const client = steamworks.init()   // reads steam_appid.txt
const client = steamworks.init(480) // explicit
```

In practice:

- **During development**, either is fine. `480` is Spacewar, the public test app id, and is what all the example scripts in this repo use. Beware that with 480 you are sharing achievements, leaderboards and lobbies with every other developer testing against it.
- **When shipping**, prefer passing your real app id in code and *not* shipping `steam_appid.txt`. When the file is present, Steam skips the check that the game was launched through Steam — handy in development, undesirable in a release build.
- Your working directory is not always your app directory (double-clicked apps, Steam-launched apps, macOS bundles). If you rely on the file, be sure of the cwd; passing the id avoids the whole question.

Read the Steamworks notes on [SteamAPI_Init](https://partner.steamgames.com/doc/api/steam_api#SteamAPI_Init).

## `restartAppIfNecessary(appId)`

Exported at module level, **not** on the client, and called *before* `init`:

```js
const steamworks = require('steamworks.js')

if (steamworks.restartAppIfNecessary(480)) {
    process.exit(0) // Steam is relaunching us; quit immediately
}

const client = steamworks.init(480)
```

If the game was started outside Steam, this asks Steam to relaunch it and returns `true`, at which point your process must exit right away without doing any further work. It returns `false` when the game was already launched through Steam, or when a `steam_appid.txt` is present (which disables the check). Details: [SteamAPI_RestartAppIfNecessary](https://partner.steamgames.com/doc/api/steam_api#SteamAPI_RestartAppIfNecessary).

Note that `restartAppIfNecessary` lives on the module, not on the object returned by `init` — always call it as `steamworks.restartAppIfNecessary(...)`, and call it before `init`.

## A complete JavaScript example

```js
const steamworks = require('steamworks.js')
const { SteamCallback } = steamworks

if (steamworks.restartAppIfNecessary(480)) {
    process.exit(0)
}

const client = steamworks.init(480)

// Synchronous calls return immediately
const me = client.localplayer.getSteamId()
console.log(`${client.localplayer.getName()} (${me.steamId64})`)
console.log('Steam Deck:', client.utils.isSteamRunningOnSteamDeck())

// Stats and achievements
client.stats.setInt('enemies_killed', 42)
client.stats.store()
client.achievement.activate('FIRST_BLOOD')

// Asynchronous calls return promises
;(async () => {
    const lobby = await client.matchmaking.createLobby(client.matchmaking.LobbyType.Public, 4)
    console.log('lobby id:', lobby.id.toString())

    lobby.setData('gamemode', 'ffa')
    lobby.leave()
})()

// Steam callbacks
const handle = client.callback.register(SteamCallback.LobbyChatUpdate, data => {
    console.log('lobby chat update', data)
})

// ... later
handle.disconnect()
```

`SteamCallback` is re-exported at module level for convenience; `client.callback.SteamCallback` is the same enum. See [[API-Callbacks]].

## A complete TypeScript example

Types ship with the package — `index.d.ts` for the module entry, `client.d.ts` for the whole native surface — so nothing extra to install.

```ts
import * as steamworks from 'steamworks.js'
// Types only. This is the pattern the repo's own smoke test uses.
import type { leaderboard, matchmaking } from 'steamworks.js/client'

const client = steamworks.init(480)

async function submitTime(ms: number): Promise<void> {
    const board = await client.leaderboard.findOrCreateLeaderboard(
        'Quickest Flag Capture',
        client.leaderboard.LeaderboardSortMethod.Ascending,
        client.leaderboard.LeaderboardDisplayType.TimeMilliSeconds
    )

    const uploaded = await board.uploadScore(
        ms,
        client.leaderboard.LeaderboardUploadScoreMethod.KeepBest
    )

    if (uploaded.scoreChanged) {
        console.log(`rank ${uploaded.globalRankPrevious} -> ${uploaded.globalRankNew}`)
    }

    const entries: leaderboard.LeaderboardEntry[] = await board.downloadEntries(
        client.leaderboard.LeaderboardDataRequest.GlobalAroundUser,
        -4,
        5,
        0
    )

    for (const entry of entries) {
        const steamId64: bigint = entry.steamId.steamId64
        console.log(entry.globalRank, entry.score, steamId64.toString())
    }
}

const filter: matchmaking.LobbyListFilter = {
    numberFilters: [{
        key: 'elo',
        value: 1500,
        comparison: client.matchmaking.LobbyNumberComparison.GreaterThanEqualTo
    }],
    slotsAvailable: 2,
    count: 20
}

void client.matchmaking.getLobbies(filter).then(lobbies => {
    for (const lobby of lobbies) {
        console.log(lobby.id.toString(), lobby.getMemberCount())
    }
})

void submitTime(12345)
```

`bigint` literals and types need `"target": "ES2020"` (or newer) in your `tsconfig.json`. The enums in `client.d.ts` are `const enum`s reachable through the client object (`client.leaderboard.LeaderboardSortMethod.Ascending`), which is how the examples above use them; importing the namespace as a *type* gives you the interfaces (`LobbyListFilter`, `LeaderboardEntry`, `WorkshopItem`, …).

If you prefer not to add a second import path, derive types from the client instead:

```ts
type Lobby = Awaited<ReturnType<typeof client.matchmaking.joinLobby>>
```

## The BigInt rule

**Every Steam id and every Steam handle is a `bigint`, never a `number`.**

They are 64-bit unsigned integers. A JavaScript `number` is a double and holds only 53 bits of integer precision, so a real `SteamID64` such as `76561199123456789` cannot be represented exactly — it silently rounds to a *different, valid-looking* id. That kind of bug survives every quick test with small numbers and then corrupts save files and matchmaking in production. The bindings use `bigint` end to end so it cannot happen.

This applies to `PlayerSteamId.steamId64`, `Lobby.id`, workshop `publishedFileId` / item ids, Steam Input action set and action handles, `Leaderboard.handle`, `FriendGame.gameId` and `lobbyId`, `cloud.FileInfo.size`, workshop statistics and progress counters.

Things that are plain `number`s: app ids, `accountId` (the 32-bit id), ranks, scores, counts, timestamps, and Steam Input action *origins*.

### Writing them

Use a `n` suffix for literals, or `BigInt(...)` for values:

```js
await client.workshop.subscribe(1234567890n)
await client.matchmaking.joinLobby(BigInt(lobbyIdFromUser))
client.networking.sendP2PPacket(peer.steamId64, client.networking.SendType.Reliable, buffer)
```

### Storing and transporting them

`bigint` does not survive `JSON.stringify` — it throws `TypeError: Do not know how to serialize a BigInt`. Convert to a string at the boundary and back on the way in:

```js
// Out: save file, HTTP body, IPC message, localStorage
const saved = { lobbyId: lobby.id.toString() }
localStorage.setItem('lastLobby', JSON.stringify(saved))

// In
const { lobbyId } = JSON.parse(localStorage.getItem('lastLobby'))
const lobby = await client.matchmaking.joinLobby(BigInt(lobbyId))
```

Same for Electron IPC: stringify ids in the main process before sending them to the renderer.

Two more sharp edges:

- **Never round-trip through `Number()`.** `BigInt(Number(id))` is exactly the precision loss you are avoiding. String is the only lossless intermediate.
- **`===` does not mix types.** `76561197960287930n === 76561197960287930` is `false` (`==` is `true`). Compare `bigint` to `bigint`, or compare `.toString()` values.

`PlayerSteamId` gives you all three forms so you can pick the right one for the job:

```js
const id = client.localplayer.getSteamId()
id.steamId64  // bigint  — the canonical 64-bit id, use this for API calls
id.steamId32  // string  — the "STEAM_1:0:1234" rendered form
id.accountId  // number  — the 32-bit account id, safe as a number
```

## Async calls and the callback pump

Calls that ask Steam for something over the network return promises: `matchmaking.createLobby` / `joinLobby` / `getLobbies`, everything on `leaderboard`, `workshop.createItem` / `updateItem` / `getItem` / `getItems` / `getAllItems` / `getUserItems` / `subscribe` / `unsubscribe` / `deleteItem`, the `auth` ticket getters, and `utils.showGamepadTextInput` / `showFloatingGamepadTextInput`.

Those promises resolve from Steam callbacks, and Steam callbacks are only delivered while `runCallbacks` is being called — which is the 30 Hz interval `init` started. Consequences worth knowing:

- **Never block the event loop.** A synchronous busy-wait on a promise deadlocks: the interval cannot fire, so the callback is never dispatched, so the promise never settles. Use `await` / `.then()`.
- **Resolution granularity is ~33 ms.** Even an instant Steam reply surfaces on the next tick of the pump. That is fine for lobbies and leaderboards; do not build a per-frame loop on it.
- **Registered callbacks** (`client.callback.register`) arrive on the same pump, in the main thread, so it is safe to touch your game state from a handler.
- `workshop.updateItemWithCallback` is the one callback-style API — it exists so you can report upload progress, which a promise cannot.
- For lowest-latency Steam Input reads, call `client.input.runFrame()` yourself right before sampling controllers instead of relying on the 30 Hz pump.

## Where to go next

Per-namespace pages, with signatures and examples:

[[API-Workshop]] · [[API-Cloud]] · [[API-Stats-and-Achievements]] · [[API-Leaderboards]] · [[API-Matchmaking]] · [[API-Friends]] · [[API-Overlay]] · [[API-Input]] · [[API-Auth]] · [[API-Networking]] · [[API-Apps-Utils-and-LocalPlayer]] · [[API-Callbacks]]

Also: [[Installation]] for Electron packaging, [[FAQ]] for common failures, [[Building-from-Source]] if you want to add a binding.
