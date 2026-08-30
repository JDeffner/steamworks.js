# API: Stats and Achievements

`client.stats` and `client.achievement` bind the per-user half of [ISteamUserStats](https://partner.steamgames.com/doc/api/ISteamUserStats): the integer and float stats your game accumulates, and the achievements it unlocks.

```ts
const steamworks = require('steamworks.js')
const client = steamworks.init(480)

client.stats.setInt('enemies_killed', 41)
client.stats.setFloat('distance_traveled', 1204.5)
client.stats.store()

client.achievement.activate('FIRST_BLOOD')
```

Both namespaces are **synchronous**. Steam keeps the user's stats in memory once the client has them, so reads and writes are local; `store()` is what pushes them to Steam's servers.

> **Everything here must be defined on the partner site first.** Stat and achievement API names are configured under your app's *Stats & Achievements* page and only exist after you publish that change. Writing an undefined stat fails silently (returns `false`), and reading one gives `null`.

> **Steam must be running.** These calls go through the initialized Steam client; calling them before `steamworks.init()` succeeded will crash the process rather than throw.

---

## Stats

Steam stats are named counters attached to the user. Each stat is declared on the partner site as either an `INT` or a `FLOAT`, and you must use the matching accessor — reading an `INT` stat with `getFloat` returns `null`.

### `getInt` / `setInt`

```ts
function getInt(name: string): number | null
function setInt(name: string, value: number): boolean
```

`getInt` returns the current value, or `null` when the stat could not be read: it is not defined for the app, it is a float stat, or the user's stats have not arrived from Steam yet.

`setInt` returns `false` on the same failure conditions. The value is a signed 32 bit integer.

```ts
const kills = client.stats.getInt('enemies_killed') ?? 0
client.stats.setInt('enemies_killed', kills + 1)
```

[`GetStat`](https://partner.steamgames.com/doc/api/ISteamUserStats#GetStat) · [`SetStat`](https://partner.steamgames.com/doc/api/ISteamUserStats#SetStat)

### `getFloat` / `setFloat`

```ts
function getFloat(name: string): number | null
function setFloat(name: string, value: number): boolean
```

The float equivalents, for stats declared as `FLOAT` or `AVGRATE` on the partner site. Same `null`/`false` failure semantics as the integer pair.

**Precision:** JS numbers are doubles, but Steam stores float stats as 32 bit floats. The value is narrowed to `f32` on the way in and widened back on the way out, so `setFloat('x', 0.1)` then `getFloat('x')` returns `0.10000000149011612`, not `0.1`. Round for display, and never use a float stat as an exact key or comparison target.

```ts
client.stats.setFloat('distance_traveled', 1204.5)
const distance: number | null = client.stats.getFloat('distance_traveled')
console.log(distance?.toFixed(1))
```

[`GetStat`](https://partner.steamgames.com/doc/api/ISteamUserStats#GetStat) · [`SetStat`](https://partner.steamgames.com/doc/api/ISteamUserStats#SetStat)

### `store`

```ts
function store(): boolean
```

Uploads every pending stat change to Steam and returns `false` if the upload could not be started.

Nothing you `setInt`/`setFloat` is durable until `store()` runs. Steam also rate-limits this call, so **do not call it every frame** — batch changes and store at a natural boundary: end of a level, on pause, before quitting.

```ts
function onLevelComplete(kills: number, meters: number) {
    client.stats.setInt('enemies_killed', kills)
    client.stats.setFloat('distance_traveled', meters)
    client.stats.store()          // one call for both
}
```

`store()` also flushes any achievement state, though `achievement.activate`/`clear` already store on your behalf.

[`StoreStats`](https://partner.steamgames.com/doc/api/ISteamUserStats#StoreStats)

### `resetAll`

```ts
function resetAll(achievementsToo: boolean): boolean
```

Resets **every** stat for this user and app to its default value. With `achievementsToo` set to `true` it also relocks every achievement.

> This wipes the player's progress for your game and cannot be undone. It exists for development and for an explicit "reset my progress" option — never call it on startup or as error recovery.

```ts
if (process.env.NODE_ENV === 'development') {
    client.stats.resetAll(true)
}
```

[`ResetAllStats`](https://partner.steamgames.com/doc/api/ISteamUserStats#ResetAllStats)

---

## Achievements

Achievements are identified by their **API Name** from the partner site (the internal string, e.g. `FIRST_BLOOD`), not by their display name.

### `activate`

```ts
function activate(achievement: string): boolean
```

Unlocks an achievement and immediately stores stats, which is what makes the Steam overlay toast pop. Returns `false` when the achievement name is unknown or the store failed.

Activating an already-unlocked achievement is harmless — it returns `true` and shows no second toast.

```ts
if (kills >= 100) client.achievement.activate('CENTURION')
```

The overlay notification only appears if the Steam overlay is available; in Electron that needs the extra setup described in [[Installation]]. The achievement still unlocks either way.

[`SetAchievement`](https://partner.steamgames.com/doc/api/ISteamUserStats#SetAchievement) · [`StoreStats`](https://partner.steamgames.com/doc/api/ISteamUserStats#StoreStats)

### `isActivated`

```ts
function isActivated(achievement: string): boolean
```

Whether the achievement is currently unlocked for this user. Returns `false` both when it is locked and when the name is unknown or the user's stats have not loaded yet — the three cases are indistinguishable, so don't use a `false` here as proof the achievement exists.

```ts
if (!client.achievement.isActivated('CENTURION') && kills >= 100) {
    client.achievement.activate('CENTURION')
}
```

[`GetAchievement`](https://partner.steamgames.com/doc/api/ISteamUserStats#GetAchievement)

### `clear`

```ts
function clear(achievement: string): boolean
```

Relocks an achievement and stores. Development and "reset progress" only — players do not expect achievements to disappear.

[`ClearAchievement`](https://partner.steamgames.com/doc/api/ISteamUserStats#ClearAchievement)

### `names`

```ts
function names(): Array<string>
```

Every achievement API name defined for this app, in the order Steam returns them. Useful for building a progress screen without hardcoding the list.

```ts
for (const name of client.achievement.names()) {
    console.log(name, client.achievement.isActivated(name) ? 'unlocked' : 'locked')
}
```

This one **throws** if Steam cannot supply the list (rather than returning an empty array), so wrap it if you call it before you are confident stats have loaded.

[`GetNumAchievements`](https://partner.steamgames.com/doc/api/ISteamUserStats#GetNumAchievements) · [`GetAchievementName`](https://partner.steamgames.com/doc/api/ISteamUserStats#GetAchievementName)

---

## Worked example

A small progress tracker that batches stat writes and unlocks tiered achievements.

```ts
import * as steamworks from 'steamworks.js'

const client = steamworks.init(480)

const KILL_TIERS: Array<[number, string]> = [
    [1, 'FIRST_BLOOD'],
    [100, 'CENTURION'],
    [1000, 'LEGION']
]

class Progress {
    private dirty = false

    addKills(n: number) {
        const total = (client.stats.getInt('enemies_killed') ?? 0) + n
        if (!client.stats.setInt('enemies_killed', total)) {
            console.warn('enemies_killed is not defined on the partner site')
            return
        }
        this.dirty = true

        for (const [threshold, name] of KILL_TIERS) {
            if (total >= threshold && !client.achievement.isActivated(name)) {
                client.achievement.activate(name)   // stores on its own
            }
        }
    }

    addDistance(meters: number) {
        const current = client.stats.getFloat('distance_traveled') ?? 0
        client.stats.setFloat('distance_traveled', current + meters)
        this.dirty = true
    }

    /** Call at a checkpoint, on pause, and before quitting — never per frame. */
    flush() {
        if (!this.dirty) return
        if (client.stats.store()) this.dirty = false
    }
}

// Achievement gallery
function gallery() {
    return client.achievement.names().map(name => ({
        name,
        unlocked: client.achievement.isActivated(name)
    }))
}
```

---

## Gotchas

- **Nothing is saved until `store()`.** Except after `achievement.activate`/`clear`, which store for you. A crash before `store()` loses the stat changes.
- **Rate limit.** Steam throttles `StoreStats`; roughly once every few seconds is safe, per-frame is not.
- **Stats may not be loaded yet.** Right after `init()` the user's stats can still be in flight, and every getter returns `null`/`false` until they arrive. Read them once gameplay starts rather than during startup, or retry.
- **Types must match the partner site.** `getFloat` on an `INT` stat returns `null` — it is not a conversion.
- **Float precision is `f32`.** See [`setFloat`](#getfloat--setfloat).
- **API names, not display names.** `FIRST_BLOOD`, not "First Blood".
- **`resetAll(true)` is destructive** and irreversible for that user.
- **No other-user stats.** This binding covers the local user only — there is no `RequestUserStats` equivalent for reading a friend's achievements.

---

## See also

- [[API-Leaderboards]] — `client.leaderboard`, the other half of ISteamUserStats
- [[API-Overlay]] — `overlay.activateDialog(Dialog.Achievements)` to open the achievement list
- [[API-Cloud]] — cloud saves, the other place per-user progress lives
- [[Getting-Started]] — `init()` semantics and app id setup
- [Stats & achievements documentation](https://partner.steamgames.com/doc/features/achievements)
