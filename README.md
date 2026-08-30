[![Build Status](https://github.com/JDeffner/steamworks.js/actions/workflows/publish.yml/badge.svg)](https://github.com/JDeffner/steamworks.js/actions/workflows/publish.yml)
[![npm](https://img.shields.io/npm/v/@jdeffner/steamworks.js.svg)](https://npmjs.com/package/@jdeffner/steamworks.js)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Chat](https://img.shields.io/discord/663831597690257431?label=chat&logo=discord)](https://discord.gg/H6B7UE7fMY)

# Steamworks.js

A modern implementation of the Steamworks SDK for HTML/JS and NodeJS based applications. Achievements, stats, leaderboards, cloud saves, workshop, lobbies, friends, P2P networking, the overlay and Steam Input — from plain JavaScript or TypeScript, with prebuilt binaries for Windows, Linux and macOS.

**Full documentation lives in the [wiki](https://github.com/JDeffner/steamworks.js/wiki)** — [installation](https://github.com/JDeffner/steamworks.js/wiki/Installation), [getting started](https://github.com/JDeffner/steamworks.js/wiki/Getting-Started), a complete API reference per module, and a [FAQ](https://github.com/JDeffner/steamworks.js/wiki/FAQ).

## About this fork

Upstream [ceifa/steamworks.js](https://github.com/ceifa/steamworks.js) last merged a change in September 2025, and its npm package has been stuck on 0.4.0 since August 2024. This fork is upstream `main` plus the reviewed pull requests that never got merged, published to npm as [`@jdeffner/steamworks.js`](https://npmjs.com/package/@jdeffner/steamworks.js), and it continues to add bindings.

The API is a **superset of upstream** — install it under the original name and every `require('steamworks.js')` keeps working, zero code changes:

```sh
npm i steamworks.js@npm:@jdeffner/steamworks.js
```

On top of everything in upstream `main`, the fork adds the Steamworks SDK upgrade via steamworks-rs v0.12.1, workshop `fileType`/permanent deletion/`returnChildren`/key-value tags/metadata/content descriptors, cloud save conflict resolution (`isFilePersisted`, `fileTimestamp`), leaderboards, the friends list with avatars, lobby list filters, and controller input glyphs. Each ported change keeps its original author in the git history; if upstream wakes up, all of it is offered back as pull requests.

## Usage

```js
const steamworks = require('steamworks.js')

// You can pass an appId, or don't pass anything and use a steam_appid.txt file
const client = steamworks.init(480)

// Print Steam username
console.log(client.localplayer.getName())

// Tries to activate an achievement
if (client.achievement.activate('ACHIEVEMENT')) {
    // ...
}
```

The [declarations file](https://github.com/JDeffner/steamworks.js/blob/main/client.d.ts) is the authoritative API surface; the [wiki's API reference](https://github.com/JDeffner/steamworks.js/wiki) documents every module with examples.

## Electron

Steamworks.js is a native module. Enable native modules in the renderer process, or keep it in the main process behind IPC (recommended — see [Installation](https://github.com/JDeffner/steamworks.js/wiki/Installation)):

```js
const mainWindow = new BrowserWindow({
    webPreferences: {
        contextIsolation: false,
        nodeIntegration: true
    }
})
```

For the Steam overlay, call this at the end of your `main.js`:

```js
require('steamworks.js').electronEnableSteamOverlay()
```

When packaging with asar, the native binaries must be unpacked or the app fails at startup with `Error: The specified module could not be found` ([#75](https://github.com/ceifa/steamworks.js/issues/75)). With electron-forge:

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

The electron-builder equivalent and more packaging details are in the [Installation](https://github.com/JDeffner/steamworks.js/wiki/Installation) wiki page.

## How to build

You **only** need to build if you are changing steamworks.js itself; to use it in a game, just install from npm. With the latest [Node.js](https://nodejs.org/en/), [Rust](https://www.rust-lang.org/tools/install) and [Clang](https://rust-lang.github.io/rust-bindgen/requirements.html) installed, run `npm ci` and `npm run build:debug`. The full development guide — generated files, the TypeScript smoke test, CI, and the manual Electron test app — is in [Building from Source](https://github.com/JDeffner/steamworks.js/wiki/Building-from-Source).
