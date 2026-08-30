# API: Leaderboards

`client.leaderboard` binds the leaderboard half of [ISteamUserStats](https://partner.steamgames.com/doc/api/ISteamUserStats): finding or creating a leaderboard, uploading the local player's score, and downloading ranked entries.

```ts
const steamworks = require('steamworks.js')
const client = steamworks.init(480)

const board = await client.leaderboard.findLeaderboard('Quickest Flag Capture')
if (board) {
    await board.uploadScore(12345, client.leaderboard.LeaderboardUploadScoreMethod.KeepBest)
}
```

Everything here is asynchronous — Steam answers over its callback pipe, which the 30 Hz `runCallbacks` interval `init()` starts pumps for you. Don't block the event loop while awaiting.

> **Steam must be running.** Calls go through the initialized Steam client; calling them before `steamworks.init()` succeeded will crash the process rather than throw.

---

## Finding a leaderboard

### `findLeaderboard`

```ts
function findLeaderboard(name: string): Promise<Leaderboard | null>
```

Looks a leaderboard up by the name configured on the Steamworks partner site.

**Resolves to `null` when no leaderboard with that name exists** — a missing board is not an error. The promise rejects only when the lookup itself failed (no connection to Steam), or when `name` contains a null byte, which is rejected up front with `Leaderboard name contains a null byte`.

```ts
const board = await client.leaderboard.findLeaderboard('Feet Traveled')
if (board === null) {
    console.log('no such leaderboard — check the partner site')
    return
}
console.log(board.getName(), board.getEntryCount())
```

Leaderboard names are case sensitive and limited by Steam to 128 characters.

[`FindLeaderboard`](https://partner.steamgames.com/doc/api/ISteamUserStats#FindLeaderboard)

### `findOrCreateLeaderboard`

```ts
function findOrCreateLeaderboard(
    name: string,
    sortMethod: LeaderboardSortMethod,
    displayType: LeaderboardDisplayType
): Promise<Leaderboard>
```

Finds a leaderboard, creating it if it does not exist. Always resolves with a `Leaderboard` or rejects — never `null`.

> **`sortMethod` and `displayType` only apply at creation.** If the leaderboard already exists it keeps whatever settings it was created with, and the arguments you pass are ignored — silently, with no indication that they differed. Read the real values back with `getSortMethod()` / `getDisplayType()` if it matters.

Leaderboards created this way show up on the partner site afterwards, where you can rename or delete them. Prefer declaring your permanent boards on the partner site and using `findLeaderboard`; keep `findOrCreateLeaderboard` for dynamic boards (per level, per daily challenge) whose names your game generates.

```ts
const board = await client.leaderboard.findOrCreateLeaderboard(
    'Quickest Flag Capture',
    client.leaderboard.LeaderboardSortMethod.Ascending,       // lower time wins
    client.leaderboard.LeaderboardDisplayType.TimeMilliSeconds
)
```

[`FindOrCreateLeaderboard`](https://partner.steamgames.com/doc/api/ISteamUserStats#FindOrCreateLeaderboard)

### `LeaderboardSortMethod`

```ts
const enum LeaderboardSortMethod {
    /** The top-score is the lowest number. */
    Ascending = 0,
    /** The top-score is the highest number. */
    Descending = 1
}
```

`Ascending` for times and stroke counts, `Descending` for points and kills.

[`ELeaderboardSortMethod`](https://partner.steamgames.com/doc/api/ISteamUserStats#ELeaderboardSortMethod)

### `LeaderboardDisplayType`

```ts
const enum LeaderboardDisplayType {
    /** The score is just a simple numerical value. */
    Numeric = 0,
    /** The score represents a time, in seconds. */
    TimeSeconds = 1,
    /** The score represents a time, in milliseconds. */
    TimeMilliSeconds = 2
}
```

This only controls how Steam formats the score in the overlay and on the community pages. Scores are always plain 32 bit integers over the wire — a `TimeMilliSeconds` board still takes `12345`, meaning 12.345 s.

[`ELeaderboardDisplayType`](https://partner.steamgames.com/doc/api/ISteamUserStats#ELeaderboardDisplayType)

---

## The `Leaderboard` class

```ts
class Leaderboard {
    get handle(): bigint
    getName(): string
    getEntryCount(): number
    getSortMethod(): LeaderboardSortMethod | null
    getDisplayType(): LeaderboardDisplayType | null
    uploadScore(
        score: number,
        method: LeaderboardUploadScoreMethod,
        details?: Array<number> | null
    ): Promise<LeaderboardScoreUploaded>
    downloadEntries(
        request: LeaderboardDataRequest,
        start: number,
        end: number,
        maxDetailsLen: number
    ): Promise<Array<LeaderboardEntry>>
}
```

A `Leaderboard` is a handle, not a snapshot — you only get one from `findLeaderboard` or `findOrCreateLeaderboard`, and you cannot construct one from a raw handle value. Hold onto the object for the session rather than looking the board up again on every score.

### `handle`

```ts
get handle(): bigint      // read-only getter
```

The raw `SteamLeaderboard_t` value, as a `bigint`. Read-only — there is no setter and no way to turn a handle back into a `Leaderboard`, so it is for logging and debugging, not for persistence.

```ts
const handle: bigint = board.handle
console.log(`leaderboard handle ${handle}`)
```

### `getName` / `getEntryCount`

```ts
getName(): string
getEntryCount(): number
```

Synchronous reads off the handle Steam already gave you. `getEntryCount()` is the total number of entries on the board — the upper bound for a `Global` download range.

```ts
console.log(`${board.getName()}: ${board.getEntryCount()} entries`)
```

[`GetLeaderboardName`](https://partner.steamgames.com/doc/api/ISteamUserStats#GetLeaderboardName) · [`GetLeaderboardEntryCount`](https://partner.steamgames.com/doc/api/ISteamUserStats#GetLeaderboardEntryCount)

### `getSortMethod` / `getDisplayType`

```ts
getSortMethod(): LeaderboardSortMethod | null
getDisplayType(): LeaderboardDisplayType | null
```

The board's actual settings, or `null` if the handle is invalid. These are how you find out what an existing leaderboard was really created with — worth checking after `findOrCreateLeaderboard`, whose arguments are ignored for a board that already exists.

```ts
if (board.getSortMethod() === client.leaderboard.LeaderboardSortMethod.Descending) {
    console.log('higher is better')
}

const display = board.getDisplayType()
const format = (score: number) =>
    display === client.leaderboard.LeaderboardDisplayType.TimeMilliSeconds
        ? `${(score / 1000).toFixed(3)}s`
        : String(score)
```

[`GetLeaderboardSortMethod`](https://partner.steamgames.com/doc/api/ISteamUserStats#GetLeaderboardSortMethod) · [`GetLeaderboardDisplayType`](https://partner.steamgames.com/doc/api/ISteamUserStats#GetLeaderboardDisplayType)

---

## Uploading a score

### `uploadScore`

```ts
uploadScore(
    score: number,
    method: LeaderboardUploadScoreMethod,
    details?: Array<number> | null
): Promise<LeaderboardScoreUploaded>
```

Uploads a score for the **local user** — you cannot write another player's entry.

- `score` is a signed 32 bit integer.
- `method` decides what happens when the user already has an entry.
- `details` is optional game-specific data stored beside the score, **at most 64 entries**, each a signed 32 bit integer. Passing more rejects the promise with `details supports at most 64 entries, got N` rather than hanging.

```ts
const enum LeaderboardUploadScoreMethod {
    /** Only replaces the existing entry if the new score is better. */
    KeepBest = 0,
    /** Always replaces the existing entry, even with a worse score. */
    ForceUpdate = 1
}
```

`KeepBest` is what you want for a personal best; Steam compares using the board's sort method, so "better" means lower on an `Ascending` board. `ForceUpdate` is for "latest run" boards and for correcting a bad upload — it will happily lower the player's rank.

```ts
interface LeaderboardScoreUploaded {
    /** The score that was submitted. */
    score: number
    /** Whether the score on the leaderboard actually changed.
     *  False when `KeepBest` was used and the existing score was better. */
    scoreChanged: boolean
    /** The new global rank of the user, 0 when the score did not change. */
    globalRankNew: number
    /** The global rank the user had before this upload, 0 when they had no entry. */
    globalRankPrevious: number
}
```

```ts
const uploaded = await board.uploadScore(
    12345,
    client.leaderboard.LeaderboardUploadScoreMethod.KeepBest,
    [seed, levelId, deaths]        // details: at most 64 int32 values
)

if (uploaded.scoreChanged) {
    const from = uploaded.globalRankPrevious === 0 ? 'unranked' : `#${uploaded.globalRankPrevious}`
    console.log(`new personal best: ${from} -> #${uploaded.globalRankNew}`)
} else {
    console.log('existing score was better, leaderboard untouched')
}
```

`details` is a good place for a replay seed, a per-segment split, or an anti-cheat checksum — anything you want back when rendering the board. Encode it as integers; there is no string details channel.

The promise rejects with `Failed to upload leaderboard score` if Steam accepted the call but returned no result, and with Steam's own error text otherwise.

[`UploadLeaderboardScore`](https://partner.steamgames.com/doc/api/ISteamUserStats#UploadLeaderboardScore) · [`LeaderboardScoreUploaded_t`](https://partner.steamgames.com/doc/api/ISteamUserStats#LeaderboardScoreUploaded_t)

---

## Downloading entries

### `downloadEntries`

```ts
downloadEntries(
    request: LeaderboardDataRequest,
    start: number,
    end: number,
    maxDetailsLen: number
): Promise<Array<LeaderboardEntry>>
```

Downloads a range of ranked entries. What `start` and `end` mean depends entirely on `request`:

```ts
const enum LeaderboardDataRequest {
    /** Query everyone on the leaderboard, `start` and `end` are absolute ranks (1 based). */
    Global = 0,
    /** Query around the current user, `start` and `end` are relative to the user's rank. */
    GlobalAroundUser = 1,
    /** Query the current user's friends, `start` and `end` are ignored. */
    Friends = 2
}
```

| Request | `start` / `end` |
| --- | --- |
| `Global` | Absolute 1-based ranks. `1, 10` is the top ten. |
| `GlobalAroundUser` | **Offsets relative to the user's own rank**, and `start` is normally negative. `-4, 5` gives the ten rows centred on the player. `0, 0` gives just the player's own row. |
| `Friends` | Ignored — pass `0, 0`. Returns the user's friends who have entries, plus the user. |

`maxDetailsLen` is how many `details` values to read back per row, **0 to 64**. Pass `0` when the board has no details; asking for more than were uploaded is harmless, you just get a shorter array.

```ts
interface LeaderboardEntry {
    /** The user that owns this entry. */
    steamId: PlayerSteamId
    /** The global rank of this entry, 1 based. */
    globalRank: number
    /** The score of this entry. */
    score: number
    /** The game specific details uploaded with the score.
     *  Empty unless `maxDetailsLen` was greater than 0 when downloading. */
    details: Array<number>
}
```

`steamId` is the usual `PlayerSteamId` (`{ steamId64: bigint, steamId32: string, accountId: number }`). To turn it into a name and avatar, use `client.friends.getFriend(entry.steamId.steamId64)` — see [[API-Friends]]; the persona may need `requestUserInformation` first for a player who is not a friend.

```ts
// Top ten
const top = await board.downloadEntries(
    client.leaderboard.LeaderboardDataRequest.Global, 1, 10, 0
)

// Ten rows around the player, with three details values each
const around = await board.downloadEntries(
    client.leaderboard.LeaderboardDataRequest.GlobalAroundUser, -4, 5, 3
)

// Friends only — start/end ignored
const friends = await board.downloadEntries(
    client.leaderboard.LeaderboardDataRequest.Friends, 0, 0, 0
)
```

If the player is near the top of the board, a `GlobalAroundUser` range clamps: asking for `-4, 5` when the player is rank 2 returns rows starting at rank 1, so you get fewer than ten rows rather than an error.

[`DownloadLeaderboardEntries`](https://partner.steamgames.com/doc/api/ISteamUserStats#DownloadLeaderboardEntries) · [`LeaderboardEntry_t`](https://partner.steamgames.com/doc/api/ISteamUserStats#LeaderboardEntry_t)

---

## Complete example

Find or create a board, upload a run, then render the ten rows around the player.

```ts
import * as steamworks from 'steamworks.js'

const client = steamworks.init(480)
const lb = client.leaderboard

async function submitRun(timeMs: number, seed: number, deaths: number) {
    // 1. Find or create. Sort/display only apply if it does not exist yet.
    const board = await lb.findOrCreateLeaderboard(
        'Quickest Flag Capture',
        lb.LeaderboardSortMethod.Ascending,          // lower time wins
        lb.LeaderboardDisplayType.TimeMilliSeconds
    )

    // Confirm what the board really is — an existing one keeps its own settings
    const ascending = board.getSortMethod() === lb.LeaderboardSortMethod.Ascending
    const asTime = board.getDisplayType() === lb.LeaderboardDisplayType.TimeMilliSeconds
    const format = (score: number) =>
        asTime ? `${(score / 1000).toFixed(3)}s` : String(score)

    console.log(`${board.getName()} — ${board.getEntryCount()} entries, ` +
        `${ascending ? 'lower' : 'higher'} is better`)

    // 2. Upload, keeping the personal best. details holds up to 64 int32 values.
    const uploaded = await board.uploadScore(
        timeMs,
        lb.LeaderboardUploadScoreMethod.KeepBest,
        [seed, deaths]
    )

    if (uploaded.scoreChanged) {
        console.log(`personal best! #${uploaded.globalRankPrevious || '—'} -> #${uploaded.globalRankNew}`)
    } else {
        console.log('previous run was faster, board unchanged')
    }

    // 3. Ten rows around the player, reading back both details values
    const me = client.localplayer.getSteamId().steamId64
    const rows = await board.downloadEntries(
        lb.LeaderboardDataRequest.GlobalAroundUser,
        -4,
        5,
        2
    )

    // 4. Render
    for (const entry of rows) {
        const isMe = entry.steamId.steamId64 === me
        const name = client.friends.getFriend(entry.steamId.steamId64).getName()
        const [ , deaths = 0 ] = entry.details

        console.log(
            `${isMe ? '>' : ' '} #${String(entry.globalRank).padStart(4)}  ` +
            `${format(entry.score).padStart(10)}  ${deaths} deaths  ${name}`
        )
    }

    return board
}

// Later, the same handle serves the other views without another lookup
async function views(board: Awaited<ReturnType<typeof submitRun>>) {
    const top10 = await board.downloadEntries(lb.LeaderboardDataRequest.Global, 1, 10, 0)
    const friends = await board.downloadEntries(lb.LeaderboardDataRequest.Friends, 0, 0, 0)
    return { top10, friends }
}
```

---

## Gotchas

- **`findLeaderboard` resolves to `null`, it does not reject,** when the board does not exist. Narrow before use.
- **`findOrCreateLeaderboard` ignores sort/display for an existing board.** Verify with `getSortMethod()` / `getDisplayType()`.
- **Scores are `int32`.** A float score has to be scaled to an integer (milliseconds, centimetres, hundredths of a point) before upload.
- **`details` caps at 64 entries**, each `int32`; oversized arrays reject the promise instead of hanging.
- **`GlobalAroundUser` wants a negative `start`.** `-4, 5` for ten rows around the player; `1, 10` there means "the ten entries starting one rank *below* the player", which is almost never what you meant.
- **Only the local user's score can be uploaded.** There is no server-side or third-party write path here.
- **Handles are not persistable.** `handle` is read-only and there is no constructor from a handle — look the board up again next session.
- **A leaderboard name with a null byte is rejected** up front rather than crashing the native side.

---

## See also

- [[API-Stats-and-Achievements]] — the other half of ISteamUserStats
- [[API-Friends]] — resolving a `PlayerSteamId` into a name and avatar for rendering rows
- [[API-Overlay]] — `overlay.activateDialog` for the Steam-side stats pages
- [[Getting-Started]] — `init()` semantics, BigInt rules, how promises are pumped
- [Leaderboards documentation](https://partner.steamgames.com/doc/features/leaderboards)
