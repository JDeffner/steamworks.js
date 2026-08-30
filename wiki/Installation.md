# Installation

You do not have to build anything. The npm package ships prebuilt native binaries for every supported platform, plus the Steam redistributable libraries they link against.

## Requirements

- **Node.js >= 14** (`engines` in `package.json`). Electron 24 or newer works out of the box; anything with a modern V8 that supports `BigInt` is fine.
- **The Steam client, installed and running, logged in.** `init()` talks to the running client over IPC. Without it, initialization throws. See [[FAQ]].
- **An app id.** Either pass it to `init(appId)` or put a `steam_appid.txt` file containing just the number next to the executable / in the working directory. During development, `480` (Spacewar) is the usual placeholder.

## Install

```sh
npm i steamworks.js@npm:@jdeffner/steamworks.js
```

This is an [npm alias install](https://docs.npmjs.com/cli/v10/commands/npm-install#description): it fetches `@jdeffner/steamworks.js` but places it in `node_modules/steamworks.js`. Your `package.json` records it as:

```json
{
    "dependencies": {
        "steamworks.js": "npm:@jdeffner/steamworks.js@^0.5.0"
    }
}
```

Because the package lands under the original folder name, `require('steamworks.js')`, `import steamworks from 'steamworks.js'`, the TypeScript types, and every bundler path resolution keep working with no code changes. This is also why packaging config that mentions `node_modules/steamworks.js/...` (see the asar section below) stays valid.

### Alternative: the scoped name

If you would rather be explicit and do not need the drop-in path, install it under its own name:

```sh
npm i @jdeffner/steamworks.js
```

Then import it as `require('@jdeffner/steamworks.js')`. You lose the drop-in property — every import and every packaging path has to be updated — so prefer the alias install unless you have a reason not to.

## Supported platforms

`index.js` picks the binary for the current `process.platform` / `process.arch`:

| Platform | Arch | Shipped binary |
| --- | --- | --- |
| `win32` | `x64` | `dist/win64/steamworksjs.win32-x64-msvc.node` |
| `linux` | `x64` | `dist/linux64/steamworksjs.linux-x64-gnu.node` |
| `darwin` | `x64` | `dist/osx/steamworksjs.darwin-x64.node` |
| `darwin` | `arm64` | `dist/osx/steamworksjs.darwin-arm64.node` |

Anything else throws at require time:

```
Error: Unsupported OS: <platform>, architecture: <arch>
```

There are no `win32-arm64`, `linux-arm64` or 32-bit builds. Windows on ARM runs the x64 build through emulation.

Next to each `.node` file the package ships the matching Steam redistributable — `steam_api64.dll` (Windows), `libsteam_api.so` (Linux), `libsteam_api.dylib` (macOS). The native module loads them from that directory, so they must stay beside the `.node` file when you package your game.

The Linux binary is built inside an `ubuntu:20.04` container to keep the glibc requirement low, so it runs on reasonably old distributions and on the Steam Deck.

## Electron

Electron needs three things sorted out: **which process calls Steam**, **the overlay switches**, and **getting the native files out of the asar archive**.

### Renderer process vs main process

steamworks.js is a native module. The renderer process cannot load native modules with Electron's defaults. You have two options.

**Option A — call Steam from the main process (recommended).** Keep `contextIsolation: true` and `nodeIntegration: false`, initialize in `main.js`, and expose only the calls your UI needs over IPC:

```js
// main.js
const { app, BrowserWindow, ipcMain } = require('electron')
const steamworks = require('steamworks.js')

const client = steamworks.init(480)

ipcMain.handle('steam:name', () => client.localplayer.getName())
ipcMain.handle('steam:achieve', (_event, name) => client.achievement.activate(name))

app.whenReady().then(() => {
    const win = new BrowserWindow({
        width: 800,
        height: 600,
        webPreferences: {
            preload: require('node:path').join(__dirname, 'preload.js')
        }
    })
    win.loadFile('index.html')
})

steamworks.electronEnableSteamOverlay()
```

```js
// preload.js
const { contextBridge, ipcRenderer } = require('electron')

contextBridge.exposeInMainWorld('steam', {
    getName: () => ipcRenderer.invoke('steam:name'),
    activate: name => ipcRenderer.invoke('steam:achieve', name)
})
```

This keeps the security defaults, keeps the 30 Hz callback pump running in one place, and avoids re-initializing Steam per window. The cost is that you hand-write an IPC wrapper for each call, and that anything returning a `bigint` or a class instance (`Lobby`, `Friend`, `Ticket`, `Leaderboard`) cannot cross IPC directly — convert to strings / plain objects in the main process.

**Option B — call Steam from the renderer.** Turn off the isolation defaults, as the working example in `test/electron/` does:

```js
const mainWindow = new BrowserWindow({
    // ...
    webPreferences: {
        // ...
        contextIsolation: false,
        nodeIntegration: true
    }
})
```

Then `require('steamworks.js')` and `steamworks.init(480)` work directly in `renderer.js`. This is the simplest path and what most existing steamworks.js games do, but it gives page scripts full Node access — only do it if you are loading local files you control, and keep a strict `Content-Security-Policy` on your pages.

Do not initialize in both processes. Pick one.

### Steam overlay

Call the helper once from `main.js`:

```js
require('steamworks.js').electronEnableSteamOverlay()
```

It does two things:

1. Appends the Chromium command line switches `in-process-gpu` and `disable-direct-composition`. Command line switches only take effect before the app is ready, so call this at module scope in `main.js` (the example calls it at the very end of the file), not inside `app.whenReady()`.
2. Unless you pass `true`, it attaches a repaint loop to every `BrowserWindow` (existing ones and any created later) that calls `webContents.invalidate()` at 60 Hz when the window is not already painting. The overlay only composites over frames the app actually renders, and an idle Electron window renders nothing — this forces a frame so the overlay stays visible.

```js
// Skip the per-frame invalidation, e.g. if your app already renders continuously
require('steamworks.js').electronEnableSteamOverlay(true)
```

The interval is stored as `browserWindow.steamworksRepaintInterval` and cleared automatically when the window is destroyed.

### asar: unpacking the native binaries

Native `.node` files and the Steam shared libraries cannot be loaded from inside an asar archive. If you do not unpack them, the packaged app fails at startup with `Error: The specified module could not be found` (Windows) or `Cannot find module ... .node` — see [ceifa/steamworks.js#75](https://github.com/ceifa/steamworks.js/issues/75).

**electron-forge** — in `forge.config.js`:

```js
module.exports = {
    packagerConfig: {
        asar: {
            unpack: '*.{node,dll,so,dylib,lib}',
            unpackDir: 'node_modules/steamworks.js/dist/**'
        }
    }
}
```

**electron-builder** — the equivalent in `electron-builder.yml` / the `build` key of `package.json` is `asarUnpack`:

```json
{
    "build": {
        "asar": true,
        "asarUnpack": [
            "**/*.{node,dll,so,dylib}",
            "node_modules/steamworks.js/dist/**"
        ]
    }
}
```

Both produce an `app.asar.unpacked/node_modules/steamworks.js/dist/<platform>/` directory next to `app.asar`, containing the `.node` file and its redistributable side by side — which is what the loader needs.

If you installed under the scoped name instead of the alias, replace `node_modules/steamworks.js` with `node_modules/@jdeffner/steamworks.js` in both configs.

After packaging, verify by listing that unpacked directory. If it is missing or contains only the `.node` without the `.dll`/`.so`/`.dylib`, the glob is wrong.

### Testing the Electron setup

The repository has a working example in `test/electron/`. From a clone:

```sh
cd test/electron
npm install
npm start
```

It initializes with app id 480, prints your Steam name, and has an "activate overlay" button that calls `client.overlay.activateToWebPage(...)`.

## Next

- [[Getting-Started]] for `init()`, app ids and the BigInt rule.
- [[FAQ]] if something already went wrong.
