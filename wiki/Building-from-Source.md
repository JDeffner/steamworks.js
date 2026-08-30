# Building from Source

You only need this if you are changing steamworks.js itself — adding a binding, porting an upstream PR, or debugging the native module. To use the library in a game, see [[Installation]]; the npm package ships prebuilt binaries for all four targets.

## Toolchain

- **Node.js** — a current LTS. CI uses 22.x.
- **Rust stable** — install via [rustup](https://www.rust-lang.org/tools/install).
- **Clang** — needed by `bindgen` to parse the Steamworks headers. See the [rust-bindgen requirements](https://rust-lang.github.io/rust-bindgen/requirements.html).
- **Steam**, installed and running, if you want to actually run anything you build.

The Steam SDK headers are vendored by [steamworks-rs](https://github.com/Noxime/steamworks-rs), which is pinned to a git rev in `Cargo.toml`. You do not download the SDK yourself.

## Build

```sh
git clone https://github.com/JDeffner/steamworks.js
cd steamworks.js
npm ci
npm run build:debug
```

`build:debug` runs `node build`, which:

1. Copies the Steam redistributable for your platform from `sdk/redistributable_bin/<folder>` into `dist/<folder>` (`steam_api64.dll` + `.lib` on Windows, `libsteam_api.so` on Linux, `libsteam_api.dylib` on macOS).
2. Runs the napi CLI to compile the Rust cdylib into `dist/<folder>/steamworksjs.<triple>.node`.
3. Regenerates `client.d.ts` at the repository root from the `#[napi]` annotations in the Rust source.

A warm rebuild is about 30 seconds. This is the loop you iterate in.

For a release build:

```sh
npm run build
```

That is `npm run types && node build --release` — it additionally regenerates `index.d.ts` from the JSDoc in `index.js` (`tsc index.js --allowJs --declaration --emitDeclarationOnly`) and compiles with optimizations and LTO. Use it before cutting a release, not for iteration.

Other scripts:

| Script | What it does |
| --- | --- |
| `npm run prune` | `rimraf dist target client.d.ts` — nuke all build output |
| `npm run format` | `cargo clippy --fix --allow-staged && cargo fmt` |
| `npm run types` | Regenerate `index.d.ts` from `index.js` JSDoc only |

You can cross-compile by passing a target through to the build script, which is what CI does:

```sh
npm run build -- --target x86_64-pc-windows-msvc
```

Windows-from-Linux needs [`cargo-xwin`](https://github.com/rust-cross/cargo-xwin) installed. Building all four targets locally is not worth setting up — push a branch and let CI do it.

## Generated files — never hand-edit

Two files in the repository are build output and are committed:

- **`client.d.ts`** — written by the napi build from the Rust source. It is the entire public API surface.
- **`index.d.ts`** — written by `tsc` from the JSDoc comments in `index.js`.

Editing either by hand does nothing useful: the next build overwrites it. To change the typed surface, change the Rust (`src/api/*.rs`) or the JSDoc in `index.js`, rebuild, and commit whatever falls out.

After a build, `git status` should show either a clean tree or a `client.d.ts` diff that exactly matches the API you meant to add. **An unexpected `client.d.ts` diff means you changed the public surface without meaning to** — that is the signal to stop and look, because this package is a drop-in replacement for upstream and existing names and signatures are frozen.

## Verifying a change

You cannot talk to Steam in CI, and you cannot assume a dev box has the Steam client running. The bar for "it works" is:

**1. The binary loads.**

```sh
node -e "require('./index.js'); console.log('ok')"
```

(Requiring the module only selects and loads the `.node` file; `init()` is what needs Steam.)

**2. The TypeScript smoke test compiles.**

```sh
cd test/typescript
npm i
npm run compile
```

This is the real integration check. `test/typescript/index.ts` is realistic typed usage of the whole surface, compiled with `tsc --noEmit --strict --target ES2020 --moduleResolution node`. Run it after any API change. If you added a binding, add usage of it to that file — that is how the new types get exercised.

**3. Lints pass.**

```sh
cargo fmt --all --check
cargo clippy
```

A `cargo fmt` diff is the single most common avoidable CI failure. Run `cargo fmt` before you push.

**4. Manual harnesses, if you have Steam running.** `test/*.js` are hand-run scripts against app id 480 (Spacewar): `user.js`, `matchmaking.js`, `networking.js`, `overlay.js`, `callback.js`, `auth.js`, `workshop.js`, `input.js`. Run them with `node test/user.js`. They are not automated tests and are not expected to pass without a live Steam client — do not write tests that require one.

**5. Electron, if you touched `index.js` or the overlay helper.**

```sh
cd test/electron
npm install
npm start
```

Click "activate overlay" to check the overlay path.

## CI and publishing

Two workflows in `.github/workflows/`:

**`typescript-smoke-test.yml`** runs on every push: installs dependencies and compiles `test/typescript`.

**`publish.yml`** runs on pushes to `main`, on `v*` tags, and on pull requests, with four jobs:

- `check` — `cargo fmt --all --check` and `cargo clippy`.
- `build` — inside an `ubuntu:20.04` container (to keep the glibc floor low) builds `x86_64-unknown-linux-gnu` and, via `cargo-xwin`, `x86_64-pc-windows-msvc`.
- `build-mac` — on `macos-latest`, builds `x86_64-apple-darwin` and `aarch64-apple-darwin`.
- `deploy` — only when a `v*` tag is pushed to `JDeffner/steamworks.js`. Downloads the artifacts from both build jobs into `dist/` and runs `npm publish --access public --provenance`.

So a release is: bump `version` in `package.json`, commit, tag `vX.Y.Z`, push the tag. The workflow builds all four platforms and publishes. Never push a tag or publish unless the maintainer asked for it.

## Conventions for changes

- Conventional commit titles in plain language: `feat(stats): leaderboard bindings`, `fix(workshop): ...`.
- **Additive changes only.** Renaming an export, changing a signature, or making an optional argument required breaks every game installed through the alias. New parameters get defaults that preserve the old behavior — `workshop.createItem`'s `fileType` defaulting to `Community` is the model.
- 64-bit values cross the boundary as `BigInt`, never `f64`. See the BigInt section in [[Getting-Started]].
- When porting an upstream pull request, keep the original author on the commit; credit the upstream PR number in the release notes.
- New public API gets a doc comment, ideally linking the relevant <https://partner.steamgames.com/doc/> page, the way the existing bindings do.
- When bumping the pinned steamworks-rs rev, refresh `sdk/redistributable_bin/` from the matching Steam SDK. A mismatch there fails at runtime on players' machines, not at build time.

The architecture of `src/` — one `#[napi] pub mod` per Steam interface, the client singleton, the `tokio::sync::oneshot` pattern for async callbacks — is documented in `CLAUDE.md` in the repository root. Read a neighboring module (`src/api/matchmaking.rs` is a good one) before adding a new binding.
