# API: Apps, Utils and LocalPlayer

Three small namespaces that answer "what is installed and owned", "what is this Steam client", and "who is the player".

```ts
const steamworks = require('steamworks.js')
const client = steamworks.init(480)

console.log(client.localplayer.getName(), client.localplayer.getSteamId().steamId64)
console.log(client.apps.currentGameLanguage(), client.utils.getAppId())
```

Everything on this page is synchronous except the two gamepad text input functions.

---

# `client.apps`

Binds [ISteamApps](https://partner.steamgames.com/doc/api/ISteamApps) — ownership, installation and app metadata.

## Ownership

```ts
function isSubscribed(): boolean
function isSubscribedApp(appId: number): boolean
function isSubscribedFromFreeWeekend(): boolean
```

- `isSubscribed()` — whether the user owns (is "subscribed to") the **current** app. In practice always `true` in a running game; it is mostly useful as a sanity check.
- `isSubscribedApp(appId)` — whether the user owns some other app, typically a DLC. [`BIsSubscribedApp`](https://partner.steamgames.com/doc/api/ISteamApps#BIsSubscribedApp)
- `isSubscribedFromFreeWeekend()` — whether the current ownership is only a free-weekend grant. [`BIsSubscribedFromFreeWeekend`](https://partner.steamgames.com/doc/api/ISteamApps#BIsSubscribedFromFreeWeekend)

> These are **client-side** answers and a modified client can lie. For anything that matters — unlocking paid content on your servers — verify server-side with an auth ticket, see [[API-Auth]].

## Installation

```ts
function isAppInstalled(appId: number): boolean
function isDlcInstalled(appId: number): boolean
function appInstallDir(appId: number): string
```

- `isAppInstalled(appId)` — whether that app is installed on this machine. [`BIsAppInstalled`](https://partner.steamgames.com/doc/api/ISteamApps#BIsAppInstalled)
- `isDlcInstalled(appId)` — whether that **DLC** app id is installed. Ownership and installation are separate: a user can own DLC they have not downloaded. [`BIsDlcInstalled`](https://partner.steamgames.com/doc/api/ISteamApps#BIsDlcInstalled)
- `appInstallDir(appId)` — the folder an app is installed in. [`GetAppInstallDir`](https://partner.steamgames.com/doc/api/ISteamApps#GetAppInstallDir)

```ts
const DLC = 1234560
if (client.apps.isSubscribedApp(DLC) && client.apps.isDlcInstalled(DLC)) {
    loadExpansionFrom(client.apps.appInstallDir(DLC))
}
```

## Build, owner and account state

```ts
function appBuildId(): number
function appOwner(): PlayerSteamId
function isVacBanned(): boolean
function isCybercafe(): boolean
function isLowViolence(): boolean
```

- `appBuildId()` — the build id of the running app, matching what the partner site shows. Handy in crash reports. [`GetAppBuildId`](https://partner.steamgames.com/doc/api/ISteamApps#GetAppBuildId)
- `appOwner()` — the steam id of the account that **owns** the app, which differs from the player when the game is Family Shared. [`GetAppOwner`](https://partner.steamgames.com/doc/api/ISteamApps#GetAppOwner)
- `isVacBanned()` — whether the user is VAC banned in this app. [`BIsVACBanned`](https://partner.steamgames.com/doc/api/ISteamApps#BIsVACBanned)
- `isCybercafe()` — whether the license is a cybercafé one.
- `isLowViolence()` — whether the user is running a low-violence build (some regions), so you can suppress gore. [`BIsLowViolence`](https://partner.steamgames.com/doc/api/ISteamApps#BIsLowViolence)

```ts
const owner = client.apps.appOwner().steamId64
const player = client.localplayer.getSteamId().steamId64
const familyShared = owner !== player
```

## Language

```ts
function currentGameLanguage(): string
function availableGameLanguages(): Array<string>
function currentBetaName(): string | null
```

- `currentGameLanguage()` — the language the user launched the game in, as a Steam [API language code](https://partner.steamgames.com/doc/store/localization/languages) (`english`, `brazilian`, `schinese`, …). Use it to pick your locale bundle. [`GetCurrentGameLanguage`](https://partner.steamgames.com/doc/api/ISteamApps#GetCurrentGameLanguage)
- `availableGameLanguages()` — every language your depot advertises. [`GetAvailableGameLanguages`](https://partner.steamgames.com/doc/api/ISteamApps#GetAvailableGameLanguages)
- `currentBetaName()` — the beta branch the user is on, or `null` on the default branch. [`GetCurrentBetaName`](https://partner.steamgames.com/doc/api/ISteamApps#GetCurrentBetaName)

```ts
const beta = client.apps.currentBetaName()
console.log(beta === null ? 'default branch' : `beta: ${beta}`)
```

---

# `client.utils`

Binds [ISteamUtils](https://partner.steamgames.com/doc/api/ISteamUtils) — facts about the running client, plus Steam's on-screen keyboards.

## `getAppId`

```ts
function getAppId(): number
```

The app id this client was initialized with. Useful when the id came from `steam_appid.txt` rather than your own code, and to compare against a friend's `getGamePlayed().appId` — see [[API-Friends]].

[`GetAppID`](https://partner.steamgames.com/doc/api/ISteamUtils#GetAppID)

## `getServerRealTime`

```ts
function getServerRealTime(): number
```

Steam's server time as **Unix epoch seconds** — a clock the player cannot set. Use it for daily resets, event windows, or anything a local clock could cheat.

```ts
const now = new Date(client.utils.getServerRealTime() * 1000)
```

[`GetServerRealTime`](https://partner.steamgames.com/doc/api/ISteamUtils#GetServerRealTime)

## `isSteamRunningOnSteamDeck`

```ts
function isSteamRunningOnSteamDeck(): boolean
```

Whether the game is running on a Steam Deck. Branch UI scale, default control scheme, or which text input to show on it.

[`IsSteamRunningOnSteamDeck`](https://partner.steamgames.com/doc/api/ISteamUtils#IsSteamRunningOnSteamDeck)

## `showGamepadTextInput`

```ts
function showGamepadTextInput(
    inputMode: GamepadTextInputMode,
    inputLineMode: GamepadTextInputLineMode,
    description: string,
    maxCharacters: number,
    existingText?: string | null
): Promise<string | null>
```

Opens Steam's full-screen Big Picture keyboard and resolves with **the entered text, or `null` if cancelled or the input could not be shown**.

```ts
const enum GamepadTextInputMode {
    Normal = 0,
    Password = 1
}

const enum GamepadTextInputLineMode {
    SingleLine = 0,
    MultipleLines = 1
}
```

[`ShowGamepadTextInput`](https://partner.steamgames.com/doc/api/ISteamUtils#ShowGamepadTextInput)

```ts
const name = await client.utils.showGamepadTextInput(
    client.utils.GamepadTextInputMode.Normal,
    client.utils.GamepadTextInputLineMode.SingleLine,
    'Enter your callsign',
    24,
    'Player'
)

if (name !== null) {
    applyCallsign(name)
}
```

Note the two `null` cases are indistinguishable: the keyboard never opened (not in Big Picture / not on a Deck), or the player dismissed it. Check `isSteamRunningOnSteamDeck()` first if you need to fall back to your own input field.

## `showFloatingGamepadTextInput`

```ts
function showFloatingGamepadTextInput(
    keyboardMode: FloatingGamepadTextInputMode,
    x: number, y: number, width: number, height: number
): Promise<boolean>
```

Opens the **floating** keyboard positioned over a screen rectangle — the one that docks beside a text field rather than taking the whole screen. Resolves `true` if the keyboard was shown, otherwise `false`.

```ts
const enum FloatingGamepadTextInputMode {
    SingleLine = 0,
    MultipleLines = 1,
    Email = 2,
    Numeric = 3
}
```

The rectangle is in **screen coordinates relative to your window**, so pass the bounds of the field you want the keyboard to avoid.

> This one does **not** give you the text. The player types into whatever field your app has focused; the promise only tells you the keyboard appeared. Read the value from your own input element.

[`ShowFloatingGamepadTextInput`](https://partner.steamgames.com/doc/api/ISteamUtils#ShowFloatingGamepadTextInput)

```ts
const rect = inputEl.getBoundingClientRect()
const shown = await client.utils.showFloatingGamepadTextInput(
    client.utils.FloatingGamepadTextInputMode.SingleLine,
    Math.round(rect.left), Math.round(rect.top),
    Math.round(rect.width), Math.round(rect.height)
)
```

---

# `client.localplayer`

The local user's identity, drawn from [ISteamUser](https://partner.steamgames.com/doc/api/ISteamUser), [ISteamFriends](https://partner.steamgames.com/doc/api/ISteamFriends) and [ISteamUtils](https://partner.steamgames.com/doc/api/ISteamUtils).

## `getSteamId`

```ts
function getSteamId(): PlayerSteamId

interface PlayerSteamId {
    steamId64: bigint
    steamId32: string   // 'STEAM_0:0:11101'
    accountId: number
}
```

The local player's id in all three common forms. `steamId64` is a `bigint` — send it to your backend as a string (`.toString()`), never as a JSON number.

[`GetSteamID`](https://partner.steamgames.com/doc/api/ISteamUser#GetSteamID)

## `getName`

```ts
function getName(): string
```

The local player's current persona name. [`GetPersonaName`](https://partner.steamgames.com/doc/api/ISteamFriends#GetPersonaName)

## `getLevel`

```ts
function getLevel(): number
```

The player's Steam community level. [`GetPlayerSteamLevel`](https://partner.steamgames.com/doc/api/ISteamUser#GetPlayerSteamLevel)

## `getIpCountry`

```ts
function getIpCountry(): string
```

The 2 digit ISO 3166-1-alpha-2 country code the client is running in, e.g. `"US"` or `"UK"`. Derived from the IP, so it is a hint (regional defaults, currency guess), not a fact — VPNs and travel exist.

[`GetIPCountry`](https://partner.steamgames.com/doc/api/ISteamUtils#GetIPCountry)

## `setRichPresence`

```ts
function setRichPresence(key: string, value?: string | null): void
```

Sets one rich presence key for the local player, visible to friends in the Steam UI and readable by your own game on their side.

**Omitting `value` (or passing `null`/`undefined`) clears that key.** There is no bound "clear everything" call — clear the keys you set.

[`SetRichPresence`](https://partner.steamgames.com/doc/api/ISteamFriends#SetRichPresence) · [Enhanced Rich Presence](https://partner.steamgames.com/doc/features/enhancedrichpresence)

### Known keys

Steam gives a few keys special meaning; everything else is your own game's data.

| Key | Meaning |
| --- | --- |
| `status` | A plain-text line shown under the player in the friends list. Superseded by `steam_display` for localized presence, but still the simplest thing that works. |
| `connect` | A command-line string a friend's client uses to join the player directly. Setting it makes "Join game" appear in the friends UI. |
| `steam_display` | A localization token from your rich presence localization file, e.g. `#Status_InMatch`. Steam renders it in the viewer's language. |
| `steam_player_group` | An id grouping players who are together, so the friends list shows them as a party. |
| `steam_player_group_size` | The size of that group, as a string. |

Steam's documented limits: at most 20 keys per user, keys up to 64 bytes and values up to 256 bytes.

```ts
// The simple version
client.localplayer.setRichPresence('status', 'In a match on Crossfire')

// Direct join: friends get a "Join game" button
const lobby = await client.matchmaking.createLobby(client.matchmaking.LobbyType.FriendsOnly, 4)
client.localplayer.setRichPresence('connect', `+connect_lobby ${lobby.id}`)

// Localized presence, with your own tokens
client.localplayer.setRichPresence('steam_display', '#Status_InMatch')
client.localplayer.setRichPresence('map', 'crossfire')       // referenced by the token
client.localplayer.setRichPresence('steam_player_group', lobby.id.toString())
client.localplayer.setRichPresence('steam_player_group_size', '4')

// Leaving the match: clear what we set
for (const key of ['status', 'connect', 'steam_display', 'map',
                   'steam_player_group', 'steam_player_group_size']) {
    client.localplayer.setRichPresence(key)
}
```

When a friend clicks "Join game", *your* process is either relaunched with that command line or — if it is already running — receives the `GameLobbyJoinRequested` callback. Handle both; see [[API-Callbacks]].

Reading *another* user's rich presence is not bound in the current surface; `client.friends` exposes state and `getGamePlayed()` instead — see [[API-Friends]].

---

## See also

- [[API-Friends]] — other users' names, states and avatars
- [[API-Auth]] — turning a client-side ownership check into something a server can trust
- [[API-Callbacks]] — `GameLobbyJoinRequested`, `MicroTxnAuthorizationResponse`
- [[API-Matchmaking]] — the lobby id that goes in `connect`
