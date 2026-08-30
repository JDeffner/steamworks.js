# API: Overlay

`client.overlay` opens the [Steam overlay](https://partner.steamgames.com/doc/features/overlay) on a particular page — the friends list, a user's profile, a store page, an invite dialog, or an arbitrary URL.

```ts
const steamworks = require('steamworks.js')
const client = steamworks.init(480)

client.overlay.activateDialog(client.overlay.Dialog.Friends)
```

Every function here is fire-and-forget: they return `void` and give you no signal about whether the overlay actually opened. If the overlay is disabled (in the user's Steam settings, or because your process does not meet the requirements below) the call is simply a no-op.

---

## `activateDialog`

```ts
function activateDialog(dialog: Dialog): void
```

Opens one of the overlay's own pages.

[`ActivateGameOverlay`](https://partner.steamgames.com/doc/api/ISteamFriends#ActivateGameOverlay)

### `Dialog`

```ts
const enum Dialog {
    Friends = 0,
    Community = 1,
    Players = 2,
    Settings = 3,
    OfficialGameGroup = 4,
    Stats = 5,
    Achievements = 6
}
```

| Value | Overlay page (the string passed to Steam) |
| --- | --- |
| `Friends` | `friends` — the friends list |
| `Community` | `community` — the game's community hub |
| `Players` | `players` — recently played with |
| `Settings` | `settings` — overlay settings |
| `OfficialGameGroup` | `officialgamegroup` — the game's official group |
| `Stats` | `stats` — the local player's stats for this game |
| `Achievements` | `achievements` — the local player's achievements |

```ts
// "View achievements" button
client.overlay.activateDialog(client.overlay.Dialog.Achievements)
```

---

## `activateDialogToUser`

```ts
function activateDialogToUser(dialog: Dialog, steamId64: bigint): void
```

Opens an overlay dialog aimed at a specific user. The steam id is a `bigint` — a `number` will corrupt ids above 2^53.

[`ActivateGameOverlayToUser`](https://partner.steamgames.com/doc/api/ISteamFriends#ActivateGameOverlayToUser)

```ts
const friend = client.friends.getFriends()[0]
client.overlay.activateDialogToUser(
    client.overlay.Dialog.Friends,
    friend.getSteamId().steamId64
)
```

Note that the `Dialog` enum here is the same one as `activateDialog`, so only the pages listed above are reachable — the per-user dialogs Steam supports that are *not* in this enum (`chat`, `steamid`, `friendadd`, …) are not bound.

---

## `activateInviteDialog`

```ts
function activateInviteDialog(lobbyId: bigint): void
```

Opens the overlay's invite dialog for a lobby, so the player can pick friends to invite.

This is the same call as `Lobby.openInviteDialog()` — use whichever reads better where you are. See [[API-Matchmaking]].

[`ActivateGameOverlayInviteDialog`](https://partner.steamgames.com/doc/api/ISteamFriends#ActivateGameOverlayInviteDialog)

```ts
const lobby = await client.matchmaking.createLobby(client.matchmaking.LobbyType.FriendsOnly, 4)
client.overlay.activateInviteDialog(lobby.id)
```

When a friend accepts, *their* game receives the `GameLobbyJoinRequested` callback with the lobby id — see [[API-Callbacks]].

---

## `activateToWebPage`

```ts
function activateToWebPage(url: string): void
```

Opens a URL in the overlay's browser.

[`ActivateGameOverlayToWebPage`](https://partner.steamgames.com/doc/api/ISteamFriends#ActivateGameOverlayToWebPage)

```ts
client.overlay.activateToWebPage('https://store.steampowered.com/app/480')
```

Good for patch notes, a wiki, or a support page without pulling the player out of the game. The URL is passed through unchanged; nothing validates the scheme.

---

## `activateToStore`

```ts
function activateToStore(appId: number, flag: StoreFlag): void
```

Opens a store page in the overlay, optionally putting the app in the player's cart on the way.

[`ActivateGameOverlayToStore`](https://partner.steamgames.com/doc/api/ISteamFriends#ActivateGameOverlayToStore)

### `StoreFlag`

```ts
const enum StoreFlag {
    None = 0,             // just show the store page
    AddToCart = 1,        // add to cart silently, no page shown
    AddToCartAndShow = 2  // add to cart and show the cart
}
```

```ts
// Show the DLC store page
client.overlay.activateToStore(1234560, client.overlay.StoreFlag.None)

// "Buy the soundtrack" button
client.overlay.activateToStore(1234570, client.overlay.StoreFlag.AddToCartAndShow)
```

`appId` is a plain 32 bit `number` here, not a `bigint`.

---

## Requirements

The overlay only works when Steam can inject into your process:

- **Steam must be running** and the game launched in a way Steam recognizes (through Steam, or with a valid `steam_appid.txt` next to the executable during development).
- **The user must have the overlay enabled** — both globally in Steam settings and for this game.
- **The rendering path must be one the overlay can hook.** This is the usual reason nothing appears.

### Electron

Electron needs two Chromium switches plus per-frame invalidation for the overlay to composite over the window. `steamworks.js` ships that as a helper:

```ts
const steamworks = require('steamworks.js')

// Before app.whenReady()
steamworks.electronEnableSteamOverlay()
```

It appends `--in-process-gpu` and `--disable-direct-composition`, and attaches a 60 Hz repaint invalidator to every `BrowserWindow` so the overlay has fresh frames to draw over. Pass `true` to skip the invalidator if you drive repaints yourself:

```ts
steamworks.electronEnableSteamOverlay(true)
```

Full setup — including where in the Electron startup this belongs and the packaging caveats — is on [[Installation]].

---

## See also

- [[Installation]] — Electron overlay setup
- [[API-Matchmaking]] — lobbies and `Lobby.openInviteDialog()`
- [[API-Friends]] — getting the steam ids to pass to `activateDialogToUser`
- [[API-Callbacks]] — `GameLobbyJoinRequested` for accepted invites
- [[API-Input]] — `Controller.showBindingPanel()` also opens through the overlay
