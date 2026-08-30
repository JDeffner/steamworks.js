# steamworks.js — community edition

Native [Steamworks SDK](https://partner.steamgames.com/doc/) bindings for Node.js and Electron games. Achievements, stats, leaderboards, cloud saves, workshop, lobbies and P2P networking, the Steam overlay, Steam Input — called directly from JavaScript or TypeScript, no C++ and no build step.

It is a native module written in Rust on top of [steamworks-rs](https://github.com/Noxime/steamworks-rs) and [napi-rs](https://napi.rs), shipped with prebuilt binaries for Windows, Linux and macOS.

## About this fork

The original [ceifa/steamworks.js](https://github.com/ceifa/steamworks.js) went dormant: its last merged change was September 2025 and the npm package has been stuck on 0.4.0 since August 2024. This fork picks it up — it is upstream `main` plus the pull requests that were reviewed but never merged, published to npm as [`@jdeffner/steamworks.js`](https://npmjs.com/package/@jdeffner/steamworks.js), and it continues to add bindings.

It is a **drop-in replacement**. The API is a superset of upstream, so you install it under the original package name with an npm alias and every `require('steamworks.js')` in your game keeps working — zero code changes. Each ported change keeps its original author in the git history, and if upstream ever revives, all of it is offered back as pull requests.

## Install

```sh
npm i steamworks.js@npm:@jdeffner/steamworks.js
```

See [[Installation]] for Electron packaging and platform details.

## Minimal usage

```js
const steamworks = require('steamworks.js')

// Pass an app id, or omit it and ship a steam_appid.txt next to your executable
const client = steamworks.init(480)

console.log(client.localplayer.getName())

if (client.achievement.activate('ACHIEVEMENT')) {
    console.log('unlocked')
}
```

`init()` throws if the Steam client is not running or the app id is wrong, and returns the API object you use for everything else. See [[Getting-Started]].

## Wiki contents

### Guides

| Page | What's in it |
| --- | --- |
| [[Installation]] | Requirements, the alias install, supported platforms, the full Electron setup (overlay switches, asar unpacking, renderer vs main process) |
| [[Getting-Started]] | `init()` semantics, `steam_appid.txt`, `restartAppIfNecessary`, the BigInt rule, how async calls are pumped |
| [[Building-from-Source]] | Rust/Node toolchain, `npm run build:debug`, generated files, the TypeScript smoke test, CI |
| [[FAQ]] | Differences from upstream, packaging errors, overlay troubleshooting, where to report issues |

### API reference

| Page | Namespace |
| --- | --- |
| [[API-Workshop]] | `client.workshop` — create, update, query, subscribe and download UGC items |
| [[API-Cloud]] | `client.cloud` — Steam Cloud file read/write, timestamps, persistence |
| [[API-Stats-and-Achievements]] | `client.achievement` and `client.stats` |
| [[API-Leaderboards]] | `client.leaderboard` — find, upload scores, download entries |
| [[API-Matchmaking]] | `client.matchmaking` — lobbies, lobby data, lobby list filters |
| [[API-Friends]] | `client.friends` — friends list, personas, avatars, game played |
| [[API-Overlay]] | `client.overlay` — dialogs, web pages, store pages, invite dialog |
| [[API-Input]] | `client.input` — controllers, action sets, action origins and glyphs |
| [[API-Auth]] | `client.auth` — session tickets and web API tickets |
| [[API-Networking]] | `client.networking` — P2P packets and sessions |
| [[API-Apps-Utils-and-LocalPlayer]] | `client.apps`, `client.utils`, `client.localplayer` |
| [[API-Callbacks]] | `client.callback` — registering Steam callbacks |

The generated [`client.d.ts`](https://github.com/JDeffner/steamworks.js/blob/main/client.d.ts) in the repository is always the authoritative surface — it is produced by the build from the Rust source, so it can never drift from what the binary actually exports.

## Links

- Repository: <https://github.com/JDeffner/steamworks.js>
- Issues: <https://github.com/JDeffner/steamworks.js/issues>
- npm: <https://npmjs.com/package/@jdeffner/steamworks.js>
- Steamworks documentation: <https://partner.steamgames.com/doc/>
