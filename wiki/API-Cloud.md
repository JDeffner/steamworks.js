# API: Cloud

`client.cloud` binds [ISteamRemoteStorage](https://partner.steamgames.com/doc/api/ISteamRemoteStorage): reading and writing the per-user, per-app files Steam Cloud syncs between a player's machines.

```ts
const steamworks = require('steamworks.js')
const client = steamworks.init(480)

if (client.cloud.isEnabledForAccount() && client.cloud.isEnabledForApp()) {
    client.cloud.writeFile('save1.json', JSON.stringify({ level: 3 }))
}

const save = JSON.parse(client.cloud.readFile('save1.json'))
```

Every function here is **synchronous** — Steam Cloud writes go to a local file first and are uploaded in the background, so nothing in this namespace returns a promise.

> **Steam must be running.** These calls go through the initialized Steam client; calling them before `steamworks.init()` succeeded will crash the process rather than throw.

---

## Enabling and status

Steam Cloud has two independent switches: the account-wide setting the user controls in the Steam client, and the per-app setting your game can toggle. Both must be on for a file to sync.

### `isEnabledForAccount`

```ts
function isEnabledForAccount(): boolean
```

Whether the user has Steam Cloud enabled globally (Steam → Settings → Cloud). Your game cannot change this; if it is `false`, tell the user rather than trying.

[`IsCloudEnabledForAccount`](https://partner.steamgames.com/doc/api/ISteamRemoteStorage#IsCloudEnabledForAccount)

### `isEnabledForApp`

```ts
function isEnabledForApp(): boolean
```

Whether Cloud is enabled for *this* app for this user — the per-game checkbox in the Steam library properties, which `setEnabledForApp` also drives.

[`IsCloudEnabledForApp`](https://partner.steamgames.com/doc/api/ISteamRemoteStorage#IsCloudEnabledForApp)

### `setEnabledForApp`

```ts
function setEnabledForApp(enabled: boolean): void
```

Turns Cloud syncing on or off for this app. Returns nothing and cannot fail from JS.

Only call this from an explicit in-game setting the player toggled. It does not override the account-wide switch: with `isEnabledForAccount()` false, files still stay local no matter what you set here.

```ts
// A "Sync saves to Steam Cloud" checkbox
function onCloudToggle(on: boolean) {
    client.cloud.setEnabledForApp(on)
}
```

[`SetCloudEnabledForApp`](https://partner.steamgames.com/doc/api/ISteamRemoteStorage#SetCloudEnabledForApp)

---

## Files

File names are flat, per-user, per-app strings — `save1.json`, `settings/keys.json`. There is no real directory API; forward slashes are just part of the name.

### `writeFile`

```ts
function writeFile(name: string, content: string): boolean
```

Writes `content` to the cloud file `name`, replacing it if it exists. Returns `false` if the write failed — most often because the file exceeds the per-file size limit, or the user's Cloud quota for your app is full.

**`content` is a UTF-8 string.** There is no binary/`Buffer` overload, so encode binary saves yourself (base64 is the usual answer) before writing.

```ts
const ok = client.cloud.writeFile('save1.json', JSON.stringify(state))
if (!ok) console.warn('cloud write failed — quota or size limit?')

// Binary payload
client.cloud.writeFile('replay.b64', replayBuffer.toString('base64'))
```

The write lands on disk immediately; the upload to Steam happens in the background and, for most apps, is finalized when the game exits. See [`isFilePersisted`](#isfilepersisted).

[`FileWrite`](https://partner.steamgames.com/doc/api/ISteamRemoteStorage#FileWrite)

### `readFile`

```ts
function readFile(name: string): string
```

Reads a cloud file and returns its contents as a UTF-8 string. **Throws** (rejects the call with `Failed to read file: ...`) when the file cannot be read — including when it does not exist and when its bytes are not valid UTF-8.

Guard it:

```ts
function loadSave(name: string): unknown | null {
    if (!client.cloud.fileExists(name)) return null
    try {
        return JSON.parse(client.cloud.readFile(name))
    } catch (err) {
        console.error('corrupt cloud save', err)
        return null
    }
}
```

[`FileRead`](https://partner.steamgames.com/doc/api/ISteamRemoteStorage#FileRead)

### `fileExists`

```ts
function fileExists(name: string): boolean
```

Whether the file exists locally for this user and app. This is the cheap check to run before `readFile`.

[`FileExists`](https://partner.steamgames.com/doc/api/ISteamRemoteStorage#FileExists)

### `deleteFile`

```ts
function deleteFile(name: string): boolean
```

Deletes the file locally **and** removes it from the Steam Cloud, returning `false` if it did not exist or could not be deleted. There is no undo — the copy on Steam's servers goes too.

[`FileDelete`](https://partner.steamgames.com/doc/api/ISteamRemoteStorage#FileDelete)

### `isFilePersisted`

```ts
function isFilePersisted(name: string): boolean
```

Whether the file is actually flagged for Cloud sync, as opposed to only existing on this machine. A file written while Cloud was disabled — for the account or for the app — exists and reads back fine but is **not** persisted.

Use it to tell the player their saves are local-only:

```ts
client.cloud.writeFile('save1.json', data)

if (!client.cloud.isFilePersisted('save1.json')) {
    showWarning('Saved locally — Steam Cloud is off for this game.')
}
```

[`FilePersisted`](https://partner.steamgames.com/doc/api/ISteamRemoteStorage#FilePersisted)

### `fileTimestamp`

```ts
function fileTimestamp(name: string): number
```

The file's last-modified time in **unix epoch seconds** (not milliseconds). Returns `0` for a file that does not exist, so check `fileExists` first if you need to distinguish "missing" from "very old".

```ts
const when = client.cloud.fileTimestamp('save1.json')
console.log(new Date(when * 1000).toLocaleString())
```

Handy for picking the newer of two save slots, or for showing "last synced" in a save browser.

[`GetFileTimestamp`](https://partner.steamgames.com/doc/api/ISteamRemoteStorage#GetFileTimestamp)

### `listFiles`

```ts
function listFiles(): Array<FileInfo>

class FileInfo {
    name: string
    size: bigint
}
```

Every cloud file for this user and app. `size` is a `bigint` byte count.

```ts
let total = 0n
for (const file of client.cloud.listFiles()) {
    total += file.size
    console.log(file.name, file.size, client.cloud.fileTimestamp(file.name))
}
console.log(`${total} bytes in the cloud`)
```

The Steamworks pair behind it, [`GetFileCount`](https://partner.steamgames.com/doc/api/ISteamRemoteStorage#GetFileCount) and [`GetFileNameAndSize`](https://partner.steamgames.com/doc/api/ISteamRemoteStorage#GetFileNameAndSize), is collapsed into one array here.

---

## Worked example: a save slot

```ts
import * as steamworks from 'steamworks.js'

const client = steamworks.init(480)

interface SaveSlot {
    name: string
    savedAt: Date
    synced: boolean
    bytes: bigint
}

function listSaves(): SaveSlot[] {
    return client.cloud
        .listFiles()
        .filter(f => f.name.endsWith('.save'))
        .map(f => ({
            name: f.name,
            savedAt: new Date(client.cloud.fileTimestamp(f.name) * 1000),
            synced: client.cloud.isFilePersisted(f.name),
            bytes: f.size
        }))
        .sort((a, b) => b.savedAt.getTime() - a.savedAt.getTime())
}

function save(slot: string, state: unknown): boolean {
    const name = `${slot}.save`
    if (!client.cloud.writeFile(name, JSON.stringify(state))) return false
    if (!client.cloud.isFilePersisted(name)) {
        console.warn(`${name} is local only — cloud is off`)
    }
    return true
}

function load(slot: string): unknown | null {
    const name = `${slot}.save`
    if (!client.cloud.fileExists(name)) return null
    try {
        return JSON.parse(client.cloud.readFile(name))
    } catch {
        return null
    }
}

function deleteSave(slot: string): boolean {
    return client.cloud.deleteFile(`${slot}.save`)
}
```

---

## Setting up Steam Cloud on the partner site

None of this works until Cloud is configured for your app — the API calls succeed but nothing ever syncs.

1. In the [Steamworks partner site](https://partner.steamgames.com/), open your app → **Application** → **Cloud**.
2. Set **Byte Quota** and **Number of Files** per user. These are the limits `writeFile` fails against; pick generously, they are per user and cheap.
3. Set the **Cloud Save Root Overrides / paths** if you also want Steam Auto-Cloud. Note that Auto-Cloud (Steam syncing folders on disk) and the ISteamRemoteStorage API used here are two different mechanisms — this namespace is the API one, and does not need path patterns configured.
4. **Publish** the change to the store. Configuration only takes effect after publishing.

Useful references: [Steam Cloud documentation](https://partner.steamgames.com/doc/features/cloud), [ISteamRemoteStorage](https://partner.steamgames.com/doc/api/ISteamRemoteStorage).

---

## Gotchas

- **Strings only.** `readFile`/`writeFile` go through UTF-8. Binary data must be encoded (base64) or it will not round-trip.
- **`readFile` throws, the rest return booleans.** Only `readFile` signals failure by throwing; `writeFile`, `deleteFile` and friends return `false`.
- **No quota API.** The library does not expose `GetQuota`, so a full quota surfaces only as `writeFile` returning `false`. Check that return value on every write.
- **`fileTimestamp` is seconds.** Multiply by 1000 before handing it to `Date`.
- **Writes are not immediately in the cloud.** `isFilePersisted` tells you the file is *marked* for sync, not that the upload has completed.
- **Deleting is permanent** and removes the server-side copy as well.

---

## See also

- [[Getting-Started]] — `init()` semantics and app id setup
- [[API-Workshop]] — publishing user content, whose `cloudFileNameFilter` query option matches cloud file names
- [[API-Stats-and-Achievements]] — the other place per-user progress lives
