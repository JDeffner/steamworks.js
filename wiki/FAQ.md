# FAQ

## How is this different from ceifa/steamworks.js, and do I need code changes?

**No code changes.** The API here is a superset of upstream: every existing export keeps its name, signature and behavior. Install it under the original package name with the npm alias and everything you already wrote keeps working:

```sh
npm i steamworks.js@npm:@jdeffner/steamworks.js
```

Compared to the `steamworks.js` package on npm (0.4.0, August 2024), version 0.5.0 adds:

- Everything merged to upstream `main` after 0.4.0 shipped: `achievement.names()`, `cloud.listFiles()`, cloud enable/disable, `Controller.getHandle()`, workshop paginated queries, `workshop.deleteItem()`.
- A Steamworks SDK upgrade through steamworks-rs 0.12.1, with refreshed redistributables ([upstream #196](https://github.com/ceifa/steamworks.js/pull/196)).
- `workshop.createItem()` taking an optional `fileType` such as `Microtransaction`, defaulting to `Community` ([upstream #191](https://github.com/ceifa/steamworks.js/pull/191)).
- `cloud.isFilePersisted()` and `cloud.fileTimestamp()` for save-sync conflict resolution ([upstream #207](https://github.com/ceifa/steamworks.js/pull/207)).
- `returnChildren` on workshop queries, with `children` and `numChildren` on results, for collections and item hierarchies.

Development continues past that — leaderboards, the friends list and rich presence, lobby list filters, and Steam Input glyphs are in the fork. Check `client.d.ts` in the repository for the current surface.

## Why did upstream stop?

It went quiet: the last change merged into ceifa/steamworks.js was in September 2025, and its npm package has not moved since 0.4.0 in August 2024. Nothing dramatic — maintainers get busy. This fork exists so the reviewed-but-unmerged work does not rot and so games have somewhere to file bugs.

## Can I go back to upstream later?

Yes, and it is the same one-line mechanism:

```sh
npm i steamworks.js@latest
```

Because you were always requiring `steamworks.js`, nothing in your code refers to the fork. The only caveat is the obvious one: if you have started using API that exists only here (leaderboards, friends, lobby filters, `cloud.fileTimestamp`, …), those calls will not exist upstream. Anything from the 0.4.0-era surface moves back cleanly.

## "Cannot find module ... .node" / "The specified module could not be found" in my packaged Electron app

The native binaries are still inside the asar archive. Node cannot load a `.node` file, or the Steam shared library next to it, from inside asar. It works in development (no asar) and breaks the moment you package — this is [upstream issue #75](https://github.com/ceifa/steamworks.js/issues/75).

Unpack them. electron-forge, in `forge.config.js`:

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

electron-builder, in your build config:

```json
{
    "asarUnpack": [
        "**/*.{node,dll,so,dylib}",
        "node_modules/steamworks.js/dist/**"
    ]
}
```

Then verify: your packaged app should contain `app.asar.unpacked/node_modules/steamworks.js/dist/<platform>/` holding *both* the `steamworksjs.*.node` file and its redistributable (`steam_api64.dll`, `libsteam_api.so`, `libsteam_api.dylib`). If the directory is missing, or holds the `.node` without the library, your glob is wrong. Full walkthrough in [[Installation]].

If you installed under the scoped name rather than the alias, the path is `node_modules/@jdeffner/steamworks.js/dist/**`.

## "Unsupported OS: ... , architecture: ..." at require time

There is no prebuilt binary for that platform/arch pair. Shipped: `win32-x64`, `linux-x64`, `darwin-x64`, `darwin-arm64`. Not shipped: Windows on ARM natively (it runs the x64 build under emulation), Linux on ARM, and any 32-bit target. See [[Building-from-Source]] if you want to compile for something else.

## The Steam overlay does not show up in my Electron app

Three things, in order:

1. **Call the helper**, at module scope in `main.js`:

   ```js
   require('steamworks.js').electronEnableSteamOverlay()
   ```

   It appends the `in-process-gpu` and `disable-direct-composition` Chromium switches. Command line switches only apply before the app is ready, so calling it inside `app.whenReady()` is too late.

2. **Do not disable hardware acceleration.** The overlay is composited onto the GPU frames your app renders. If you call `app.disableHardwareAcceleration()`, or you are running in a VM / remote desktop / headless environment with no GPU, there is nothing for Steam to draw on and no amount of configuration fixes it.

3. **Leave the per-frame invalidation on.** By default the helper attaches a 60 Hz loop to every window that calls `webContents.invalidate()` when the window is not already painting. An idle Electron window renders no frames at all, so the overlay would have nothing to composite over — that loop forces a frame. If your app already renders continuously (a canvas/WebGL game loop) and you would rather not pay for the extra repaints, opt out:

   ```js
   require('steamworks.js').electronEnableSteamOverlay(true)
   ```

Also check that the game was launched through Steam (or has a `steam_appid.txt` present) and that the overlay is enabled both globally in Steam's settings and per-game in the game's properties. A pure Node script has no graphical context, so `overlay.activateToWebPage(...)` there just foregrounds the Steam client instead — that is expected, and is what `test/overlay.js` notes.

## Does this work without Steam running?

No. `init()` throws — Steam not running, not logged in, unknown app id, or a missing app id all end the same way. There is no offline mode; the library is a thin binding over the Steamworks API, which is IPC to the running Steam client.

If your game should also run outside Steam (an itch.io build, a dev harness), guard initialization and treat Steam as an optional service:

```js
let client = null
try {
    client = require('steamworks.js').init(480)
} catch {
    console.warn('Steam not available; achievements disabled')
}

// later
client?.achievement.activate('FIRST_BLOOD')
```

## Why `bigint` instead of `number`?

Steam ids and handles are 64-bit unsigned integers. A JavaScript `number` is a double, exact only up to 2^53, so a real `SteamID64` like `76561199123456789` cannot be represented — it rounds to a *different but perfectly plausible* id. Nothing throws; you just get the wrong player, the wrong lobby, or the wrong workshop item, and only for some users, and only in production.

Using `bigint` makes that impossible. The price is that `JSON.stringify` refuses to serialize it, so convert at the boundary:

```js
const stored = lobby.id.toString()          // out
const lobby = await client.matchmaking.joinLobby(BigInt(stored)) // in
```

Never round-trip through `Number()` — that is exactly the precision loss you are avoiding. Details and the full list of which fields are `bigint` are in [[Getting-Started]].

## My Node script never exits

`init()` starts a 30 Hz `setInterval` to pump Steam callbacks, and a live timer keeps the Node event loop alive. There is no client shutdown binding. Call `process.exit()` when your script is done. This does not affect Electron or game apps, which have their own lifecycle.

## Can I use it with `contextIsolation: true`?

Yes — initialize in the **main** process and expose the calls you need over IPC through a preload script. That is the recommended setup. The alternative, `contextIsolation: false` plus `nodeIntegration: true` so the renderer can `require` the native module directly, is what the bundled example does and what most existing games do; it is simpler but gives page scripts full Node access. Both are laid out in [[Installation]].

Remember that `bigint` values and class instances (`Lobby`, `Friend`, `Ticket`, `Leaderboard`) cannot cross IPC — convert them to strings and plain objects on the main side.

## Which app id should I use in development?

`480` (Spacewar) is the public test app, and every example script in the repository uses it. It works without owning anything. Be aware that its achievements, leaderboards and lobbies are shared with every other developer testing against it, so anything you write there is public and noisy. Switch to your real app id as soon as you have one.

## Where do I report issues, and do fixes go upstream?

File issues and pull requests on this fork: <https://github.com/JDeffner/steamworks.js/issues>. That is where the maintenance actually happens.

Everything here is kept in a shape that can be offered back — each ported change keeps its original author in the git history — and if ceifa/steamworks.js revives, the work is submitted upstream as pull requests. Contributing here does not fork the community; it parks the change somewhere it will be maintained in the meantime.
