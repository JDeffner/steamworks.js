[![Build Status](https://github.com/JDeffner/steamworks.js/actions/workflows/publish.yml/badge.svg)](https://github.com/JDeffner/steamworks.js/actions/workflows/publish.yml)
[![npm](https://img.shields.io/npm/v/@jdeffner/steamworks.js.svg)](https://npmjs.com/package/@jdeffner/steamworks.js)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Chat](https://img.shields.io/discord/663831597690257431?label=chat&logo=discord)](https://discord.gg/H6B7UE7fMY)

# Steamworks.js

A modern implementation of the Steamworks SDK for HTML/JS and NodeJS based applications.

## About this fork

Upstream [ceifa/steamworks.js](https://github.com/ceifa/steamworks.js) last merged a change in September 2025, and its npm package has been stuck on 0.4.0 since August 2024. This fork is upstream `main` plus reviewed pull requests that never got merged, published to npm as [`@jdeffner/steamworks.js`](https://npmjs.com/package/@jdeffner/steamworks.js).

The API is a superset of upstream. Install it under the original name and your imports keep working:

```sh
npm i steamworks.js@npm:@jdeffner/steamworks.js
```

The package is also on [GitHub Packages](https://github.com/JDeffner/steamworks.js/pkgs/npm/steamworks.js). To install from there instead of npmjs.org, point the `@jdeffner` scope at the GitHub registry in your project's `.npmrc` and authenticate with a GitHub token that has the `read:packages` scope:

```ini
@jdeffner:registry=https://npm.pkg.github.com
//npm.pkg.github.com/:_authToken=${GITHUB_TOKEN}
```

Compared to the `steamworks.js` package on npm (0.4.0), this fork adds:

* Everything merged to upstream `main` after the 0.4.0 release: `achievement.names()`, `cloud.listFiles()`, cloud enable/disable, `Controller.getHandle()`, workshop paginated queries, and `workshop.deleteItem()`
* Steamworks SDK upgrade through [steamworks-rs](https://github.com/Noxime/steamworks-rs) v0.12.1 with refreshed redistributables ([#196](https://github.com/ceifa/steamworks.js/pull/196))
* `workshop.createItem()` takes an optional `fileType` such as `Microtransaction`, with `Community` as the default ([#191](https://github.com/ceifa/steamworks.js/pull/191))
* `cloud.isFilePersisted()` and `cloud.fileTimestamp()` for save-sync conflict resolution ([#207](https://github.com/ceifa/steamworks.js/pull/207))
* `returnChildren` on workshop queries, with `children` and `numChildren` on results, for collections and item hierarchies

Each change keeps its original author in the git history. If upstream wakes up, all of this is offered back as pull requests.

## Why

I used [greenworks](https://github.com/greenheartgames/greenworks) for a long time and it's great, but I gave up for the following reasons.

* It's not being maintained anymore.
* It's not up to date.
* It's not context-aware.
* You have to build the binaries by yourself.
* Don't have typescript definitions.
* The API it's not trustful.
* The API implement callbacks instead of return flags or promises.
* I hate C++.

## API

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

You can refer to the [declarations file](https://github.com/ceifa/steamworks.js/blob/main/client.d.ts) to check the API support and get more detailed documentation of each function.

## Installation

To use steamworks.js you don't have to build anything, just install it from npm:

```sh
$: npm i steamworks.js@npm:@jdeffner/steamworks.js
```

This installs the fork under the name `steamworks.js`, so `require('steamworks.js')` works unchanged.

### Electron

Steamworks.js is a native module and cannot be used by default in the renderer process. To enable the usage of native modules on the renderer process, the following configurations should be made on `main.js`:

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

To make the steam overlay working, call the `electronEnableSteamOverlay` on the end of your `main.js` file:

```js
require('steamworks.js').electronEnableSteamOverlay()
```

For the production build, copy the relevant distro files from `sdk/redistributable_bin/{YOUR_DISTRO}` into the root of your build.

If you are using electron-forge with asar, the native binaries must be unpacked from the archive or the app fails at startup with `Error: The specified module could not be found` ([#75](https://github.com/ceifa/steamworks.js/issues/75)). Add this to `forge.config.js`:

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


## How to build

> You **only** need to build if you are going to change something on steamworks.js code, if you are looking to just consume the library or use it in your game, refer to the [installation section](#installation).

Make sure you have the latest [node.js](https://nodejs.org/en/), [Rust](https://www.rust-lang.org/tools/install) and [Clang](https://rust-lang.github.io/rust-bindgen/requirements.html). We also need [Steam](https://store.steampowered.com/about/) installed and running.

Install dependencies with `npm install` and then run `npm run build:debug` to build the library.

There is no way to build for all targets easily. The good news is that you don't need to. You can develop and test on your current target, and open a PR. When the code is merged to main, a github action will build for all targets and publish a new version.

### Testing Electron

Go to the [test/electron](./test/electron) directory. There, you can run `npm install` and then `npm start` to run the Electron app.

Click "activate overlay" to test the overlay.
