# API: Input

`client.input` binds [ISteamInput](https://partner.steamgames.com/doc/api/ISteamInput): Steam's controller abstraction, where you read named *actions* ("jump", "move") instead of raw buttons and Steam maps them to whatever the player is holding.

```ts
const steamworks = require('steamworks.js')
const client = steamworks.init(480)

client.input.init()

const inGame = client.input.getActionSet('InGameControls')
const jump = client.input.getDigitalAction('jump')

for (const controller of client.input.getControllers()) {
    controller.activateActionSet(inGame)
    if (controller.isDigitalActionPressed(jump)) {
        console.log('jump!')
    }
}
```

**All handles are `bigint`** — controller handles, action set handles, digital and analog action handles. They are 64 bit values from Steam and must never round-trip through `number`.

Action *origins*, by contrast, are plain `number`s (see [Origins and glyphs](#origins-and-glyphs)).

---

## Lifecycle

### `init`

```ts
function init(): void
```

Initializes the Steam Input API. Call it once after `steamworks.init()` and before anything else in this namespace.

Steam Input is initialized in non-explicit-call mode, meaning Steam pumps input for you as callbacks run; you do not have to drive it yourself (but see [`runFrame`](#runframe)).

[`Init`](https://partner.steamgames.com/doc/api/ISteamInput#Init)

### `shutdown`

```ts
function shutdown(): void
```

Shuts the Steam Input API down. Call it before exiting; the controller and action handles you are holding become meaningless afterwards.

[`Shutdown`](https://partner.steamgames.com/doc/api/ISteamInput#Shutdown)

### `runFrame`

```ts
function runFrame(): void
```

Synchronizes the API state with the latest Steam Input action data.

This happens automatically while callbacks are running — `steamworks.init()` starts a 30 Hz `runCallbacks` interval for you — so you only need to call it directly to shave latency, right before reading controller state in a frame.

[`RunFrame`](https://partner.steamgames.com/doc/api/ISteamInput#RunFrame)

```ts
function gameLoop() {
    client.input.runFrame()          // freshest possible state
    const pressed = controller.isDigitalActionPressed(jump)
    // …
}
```

### `setInputActionManifestFilePath`

```ts
function setInputActionManifestFilePath(path: string): boolean
```

Loads a specific action manifest (`.vdf`) from disk instead of the one configured on the Steamworks partner site. Returns `false` on failure.

Useful during development, when your `game_actions_<appid>.vdf` is not yet published. Call it after `init()`.

[`SetInputActionManifestFilePath`](https://partner.steamgames.com/doc/api/ISteamInput#SetInputActionManifestFilePath)

```ts
client.input.init()
if (!client.input.setInputActionManifestFilePath('./game_actions_480.vdf')) {
    console.warn('could not load the action manifest')
}
```

---

## Controllers

### `getControllers`

```ts
function getControllers(): Array<Controller>
```

All currently connected controllers Steam Input knows about. Returns an empty array when none are connected — poll it, controllers come and go.

[`GetConnectedControllers`](https://partner.steamgames.com/doc/api/ISteamInput#GetConnectedControllers)

### `Controller`

```ts
class Controller {
    activateActionSet(actionSetHandle: bigint): void
    isDigitalActionPressed(actionHandle: bigint): boolean
    getAnalogActionVector(actionHandle: bigint): AnalogActionVector
    getType(): InputType
    getHandle(): bigint
    getDigitalActionOrigins(actionSetHandle: bigint, actionHandle: bigint): Array<number>
    getAnalogActionOrigins(actionSetHandle: bigint, actionHandle: bigint): Array<number>
    showBindingPanel(): boolean
}
```

`getHandle()` returns the controller's own `InputHandle_t`, useful as a stable key when you track controllers across polls.

### `InputType`

```ts
getType(): InputType
```

A **string** enum — the values are the names themselves, so `controller.getType() === 'PS5Controller'` works.

```ts
const enum InputType {
    Unknown = 'Unknown',
    SteamController = 'SteamController',
    XBox360Controller = 'XBox360Controller',
    XBoxOneController = 'XBoxOneController',
    GenericGamepad = 'GenericGamepad',
    PS4Controller = 'PS4Controller',
    AppleMFiController = 'AppleMFiController',
    AndroidController = 'AndroidController',
    SwitchJoyConPair = 'SwitchJoyConPair',
    SwitchJoyConSingle = 'SwitchJoyConSingle',
    SwitchProController = 'SwitchProController',
    MobileTouch = 'MobileTouch',
    PS3Controller = 'PS3Controller',
    PS5Controller = 'PS5Controller',
    SteamDeckController = 'SteamDeckController'
}
```

Prefer glyphs from `getGlyphForActionOrigin` over branching on `getType()` for button prompts — origins already account for the player's rebinds, the controller type does not.

[`GetInputTypeForHandle`](https://partner.steamgames.com/doc/api/ISteamInput#GetInputTypeForHandle)

---

## Action sets and actions

Handles are looked up by the names in your action manifest. Look them up **once** at startup and keep them — they are stable for the lifetime of the Steam Input session.

```ts
function getActionSet(actionSetName: string): bigint
function getDigitalAction(actionName: string): bigint
function getAnalogAction(actionName: string): bigint
```

- [`GetActionSetHandle`](https://partner.steamgames.com/doc/api/ISteamInput#GetActionSetHandle)
- [`GetDigitalActionHandle`](https://partner.steamgames.com/doc/api/ISteamInput#GetDigitalActionHandle)
- [`GetAnalogActionHandle`](https://partner.steamgames.com/doc/api/ISteamInput#GetAnalogActionHandle)

A name Steam does not recognize (typo, manifest not loaded yet) yields a handle of `0n` rather than an error — check for it if you want a loud failure:

```ts
const jump = client.input.getDigitalAction('jump')
if (jump === 0n) {
    throw new Error('no "jump" action in the action manifest')
}
```

### `activateActionSet`

```ts
controller.activateActionSet(actionSetHandle: bigint): void
```

Switches the controller to an action set — menu controls versus in-game controls, driving versus on-foot. Only the active set's actions report state.

[`ActivateActionSet`](https://partner.steamgames.com/doc/api/ISteamInput#ActivateActionSet)

### Reading digital actions

```ts
controller.isDigitalActionPressed(actionHandle: bigint): boolean
```

`true` while the action is held. This is the `bState` of Steam's digital action data; the binding does not expose `bActive`, so an action that is not bound in the current action set simply reads `false`.

[`GetDigitalActionData`](https://partner.steamgames.com/doc/api/ISteamInput#GetDigitalActionData)

### Reading analog actions

```ts
controller.getAnalogActionVector(actionHandle: bigint): AnalogActionVector

interface AnalogActionVector {
    x: number
    y: number
}
```

The x/y of the analog action. For a stick that is roughly `-1..1` per axis; for a mouse-like or trackpad source the units follow the action's mode in the manifest. As with digital actions, `bActive` and the source mode are not exposed — an unbound action reads `{ x: 0, y: 0 }`.

[`GetAnalogActionData`](https://partner.steamgames.com/doc/api/ISteamInput#GetAnalogActionData)

---

## Origins and glyphs

New in 0.6. An *origin* is the physical input a player's binding currently maps an action to — "the A button", "the left trackpad". Origins are what you show in on-screen prompts, because they follow the player's own rebinds.

### `getDigitalActionOrigins` / `getAnalogActionOrigins`

```ts
controller.getDigitalActionOrigins(actionSetHandle: bigint, actionHandle: bigint): Array<number>
controller.getAnalogActionOrigins(actionSetHandle: bigint, actionHandle: bigint): Array<number>
```

The origin(s) this controller currently binds the action to, within the given action set. An action can be bound to several inputs, hence an array; an unbound action gives you an empty array.

Each entry is the **numeric value of an [`EInputActionOrigin`](https://partner.steamgames.com/doc/api/ISteamInput#EInputActionOrigin)** — a plain `number`, not a bound enum. The values are Valve's; look them up in the linked SDK docs if you need to special-case one.

- [`GetDigitalActionOrigins`](https://partner.steamgames.com/doc/api/ISteamInput#GetDigitalActionOrigins)
- [`GetAnalogActionOrigins`](https://partner.steamgames.com/doc/api/ISteamInput#GetAnalogActionOrigins)

### `getGlyphForActionOrigin`

```ts
function getGlyphForActionOrigin(origin: number): string
```

Returns the **local file path of a PNG** on disk that Steam ships for that origin — not a URL, not image bytes. Load it with `fs`, or point an `<img src>` at it with a `file://` URL.

Throws if `origin` is not a valid `EInputActionOrigin` value, so only pass numbers that came out of an origins getter.

[`GetGlyphForActionOrigin`](https://partner.steamgames.com/doc/api/ISteamInput#GetGlyphForActionOrigin)

### `getStringForActionOrigin`

```ts
function getStringForActionOrigin(origin: number): string
```

The localized, human readable name of the origin — `"A Button"`, `"Left Trigger"` — for text prompts and accessibility labels. Same validation as above.

[`GetStringForActionOrigin`](https://partner.steamgames.com/doc/api/ISteamInput#GetStringForActionOrigin)

### `showBindingPanel`

```ts
controller.showBindingPanel(): boolean
```

Opens the Steam overlay's binding panel for this controller so the player can rebind. Returns `false` if the overlay is unavailable — in Electron that usually means the overlay is not hooked, see [[API-Overlay]] and [[Installation]].

[`ShowBindingPanel`](https://partner.steamgames.com/doc/api/ISteamInput#ShowBindingPanel)

```ts
if (!controller.showBindingPanel()) {
    console.log('could not open the binding panel — is the Steam overlay available?')
}
```

---

## Complete glyph-rendering example

Building a prompt row — icon plus label — for every input the player has bound to `jump` and `move`:

```ts
import { promises as fs } from 'fs'

const steamworks = require('steamworks.js')
const client = steamworks.init(480)

client.input.init()
client.input.setInputActionManifestFilePath('./game_actions_480.vdf')
client.input.runFrame()

const actionSet: bigint = client.input.getActionSet('InGameControls')
const jump: bigint = client.input.getDigitalAction('jump')
const move: bigint = client.input.getAnalogAction('move')

interface Prompt {
    label: string
    glyphPath: string
    glyphDataUrl: string
}

async function promptsFor(controller: import('steamworks.js/client').input.Controller): Promise<Prompt[]> {
    const origins = [
        ...controller.getDigitalActionOrigins(actionSet, jump),
        ...controller.getAnalogActionOrigins(actionSet, move)
    ]

    const prompts: Prompt[] = []
    for (const origin of origins) {
        const glyphPath = client.input.getGlyphForActionOrigin(origin)
        const label = client.input.getStringForActionOrigin(origin)

        // The glyph is a PNG on disk; inline it for a renderer process
        const png = await fs.readFile(glyphPath)
        prompts.push({
            label,
            glyphPath,
            glyphDataUrl: `data:image/png;base64,${png.toString('base64')}`
        })
    }
    return prompts
}

for (const controller of client.input.getControllers()) {
    controller.activateActionSet(actionSet)
    console.log(controller.getType(), await promptsFor(controller))
}

// On exit
client.input.shutdown()
```

Rebuild the prompts when the player changes their bindings — after `showBindingPanel()` closes, or simply whenever the controller set changes.

### Gotchas

- **Origins go stale.** They reflect the *current* binding; re-query after the player rebinds rather than caching for the session.
- **`getGlyphForActionOrigin` throws on a bad number.** Anything outside `EInputActionOrigin` is rejected rather than silently transmuted, so wrap hand-written origin numbers in a `try`.
- **Handles are `bigint`, origins are `number`.** Mixing them up is a `TypeError` at the boundary, not a silent bug — but the compiler catches it first if you keep the types.
- **`init()` before anything else**, and `shutdown()` before exit.

---

## See also

- [[API-Overlay]] — the overlay the binding panel opens through
- [[Installation]] — Electron overlay requirements
- [[API-Apps-Utils-and-LocalPlayer]] — `utils.isSteamRunningOnSteamDeck()` and the gamepad text input
