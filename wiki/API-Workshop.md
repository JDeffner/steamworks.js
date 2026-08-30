# API: Workshop

`client.workshop` binds [ISteamUGC](https://partner.steamgames.com/doc/api/ISteamUGC): publishing and updating Steam Workshop items, subscribing to and downloading them, and querying the workshop catalogue.

```ts
const steamworks = require('steamworks.js')
const client = steamworks.init(480)

const { itemId } = await client.workshop.createItem()
await client.workshop.updateItem(itemId, {
    title: 'My first item',
    contentPath: '/absolute/path/to/content',
    changeNote: 'Initial upload'
})
```

> **Every id is a `bigint`.** Published file ids are 64 bit and silently lose precision as a JS `number` — write them as `123n`, and never round-trip them through `JSON.parse`/`Number`.

> **Steam must be running.** Every call goes through the initialized client. The promises here resolve from Steam callbacks, pumped by the 30 Hz `runCallbacks` interval `init()` starts for you — don't block the event loop while awaiting them.

Contents:

- [Publishing](#publishing) — `createItem`, `updateItem`, `updateItemWithCallback`, the `UgcUpdate` shape
- [Consuming](#consuming) — subscribe, download, install info, item state
- [Querying](#querying) — `getItem`, `getItems`, `getAllItems`, `getUserItems`
- [Deleting](#deleting) — `deleteItem`
- [End-to-end example](#end-to-end-example)

---

## Publishing

### `createItem`

```ts
function createItem(
    appId?: number | null,
    fileType?: FileType | null
): Promise<UgcResult>
```

Reserves a new, empty workshop item and resolves with its id. The item exists on Steam immediately but has no title, description or content until you call `updateItem` on it — it is invisible in the workshop until then.

- `appId` — the app the item belongs to. Defaults to the app id the client was initialized with.
- `fileType` — defaults to `FileType.Community`, which is what a normal player-published mod is.

```ts
interface UgcResult {
    itemId: bigint
    needsToAcceptAgreement: boolean
}
```

`needsToAcceptAgreement` is `true` when the user has not yet accepted the [Steam Workshop Legal Agreement](https://steamcommunity.com/sharedfiles/workshoplegalagreement). The item is still created, but it will not be visible publicly until they accept — send them to the agreement page (for example with `client.overlay.activateToWebPage(...)`, see [[API-Overlay]]).

[`CreateItem`](https://partner.steamgames.com/doc/api/ISteamUGC#CreateItem)

#### `FileType`

```ts
const enum FileType {
    Community = 0,
    Microtransaction = 1,
    Collection = 2,
    Art = 3,
    Video = 4,
    Screenshot = 5,
    Game = 6,
    Software = 7,
    Concept = 8,
    WebGuide = 9,
    IntegratedGuide = 10,
    Merch = 11,
    ControllerBinding = 12,
    SteamworksAccessInvite = 13,
    SteamVideo = 14,
    GameManagedItem = 15
}
```

For player-created mods you want `Community` (the default). `Collection` creates a collection rather than an item. `Microtransaction` and `GameManagedItem` are for items your game or backend owns rather than the user; several of the remaining values are internal Steam types you cannot publish from a game client.

```ts
// Explicit type and app id
const collection = await client.workshop.createItem(480, client.workshop.FileType.Collection)
```

The `fileType` parameter is additive: omitting it behaves exactly as older versions of the library did.

### `updateItem`

```ts
function updateItem(
    itemId: bigint,
    updateDetails: UgcUpdate,
    appId?: number | null
): Promise<UgcResult>
```

Applies an update to an existing item and resolves once Steam has committed it. Everything in `UgcUpdate` is optional — fields you omit are left untouched on the item. Rejects with Steam's error text on failure.

Uploading content can take a long time (it is a full file transfer); use `updateItemWithCallback` if you want progress.

[`StartItemUpdate`](https://partner.steamgames.com/doc/api/ISteamUGC#StartItemUpdate) · [`SubmitItemUpdate`](https://partner.steamgames.com/doc/api/ISteamUGC#SubmitItemUpdate)

### `UgcUpdate`

```ts
interface UgcUpdate {
    title?: string
    description?: string
    changeNote?: string
    previewPath?: string
    contentPath?: string
    tags?: Array<string>
    visibility?: UgcItemVisibility
    metadata?: string
    keyValueTags?: Array<KeyValueTag>
    removeKeyValueTags?: Array<string>
    removeAllKeyValueTags?: boolean
    contentDescriptors?: Array<ContentDescriptor>
    removeContentDescriptors?: Array<ContentDescriptor>
    allowAdminTags?: boolean
}
```

| Field | Notes |
| --- | --- |
| `title` | Item title as shown in the workshop |
| `description` | Item description, workshop-flavoured BBCode is allowed |
| `changeNote` | The changelog entry for *this* update |
| `previewPath` | **Absolute** path to the preview image (jpg/png/gif, 1 MB max) |
| `contentPath` | **Absolute** path to the *folder* holding the item's content |
| `tags` | Replaces the item's whole tag list |
| `visibility` | `UgcItemVisibility` |
| `metadata` | Developer-defined metadata, up to 5000 bytes, never shown to users |
| `keyValueTags` | Key/value tags to add |
| `removeKeyValueTags` | Keys to remove |
| `removeAllKeyValueTags` | Clear every key/value tag first |
| `contentDescriptors` | Mature-content descriptors to add |
| `removeContentDescriptors` | Mature-content descriptors to remove |
| `allowAdminTags` | Permit admin-only tags in `tags`; defaults to `false` |

`contentPath` and `previewPath` must be paths on the local disk that Steam can read — relative paths resolve against the process working directory, which in a packaged Electron app is rarely what you expect, so build absolute paths.

#### `UgcItemVisibility`

```ts
const enum UgcItemVisibility {
    Public = 0,
    FriendsOnly = 1,
    Private = 2,
    Unlisted = 3
}
```

An item created with `createItem` is not visible in the workshop until an update has been submitted for it, so set `visibility` explicitly on the first update that uploads content. Setting it to `Private` or `Unlisted` later is the non-destructive alternative to [`deleteItem`](#deleteitem).

#### Metadata

```ts
metadata?: string
```

An opaque, developer-only string attached to the item — Steam never shows it to users. Up to 5000 bytes. It is the natural place for a schema version, a dependency list, or anything your loader needs before it downloads the content.

```ts
await client.workshop.updateItem(itemId, {
    metadata: JSON.stringify({ schema: 2, requires: ['core'], entry: 'main.lua' })
})
```

Read it back with `includeMetadata: true` on a query — see [`WorkshopItemQueryConfig`](#workshopitemqueryconfig).

[`SetItemMetadata`](https://partner.steamgames.com/doc/api/ISteamUGC#SetItemMetadata)

#### Key/value tags

```ts
interface KeyValueTag {
    key: string
    value: string
}
```

Key/value tags are searchable structured tags on an item. Steam allows **up to 100 per item**, and the same key may appear more than once with different values (so `category=weapon` and `category=melee` can coexist).

Unlike `tags`, the key/value set is edited incrementally, and the three fields are applied in a fixed order inside one update:

1. `removeAllKeyValueTags: true` — clears everything
2. `removeKeyValueTags: [...]` — removes all pairs under each listed key
3. `keyValueTags: [...]` — adds the new pairs

Because removals run first, a **full replace of the tag set is one call**:

```ts
await client.workshop.updateItem(itemId, {
    removeAllKeyValueTags: true,
    keyValueTags: [
        { key: 'category', value: 'weapon' },
        { key: 'difficulty', value: 'hard' }
    ],
    changeNote: 'Retag'
})
```

Or surgically, keeping everything else:

```ts
await client.workshop.updateItem(itemId, {
    removeKeyValueTags: ['deprecated'],
    keyValueTags: [{ key: 'schema', value: '2' }]
})
```

[`AddItemKeyValueTag`](https://partner.steamgames.com/doc/api/ISteamUGC#AddItemKeyValueTag) · [`RemoveItemKeyValueTags`](https://partner.steamgames.com/doc/api/ISteamUGC#RemoveItemKeyValueTags) · [`RemoveAllItemKeyValueTags`](https://partner.steamgames.com/doc/api/ISteamUGC#RemoveAllItemKeyValueTags)

#### Content descriptors

```ts
const enum ContentDescriptor {
    /** Some Nudity or Sexual Content */
    NudityOrSexualContent = 0,
    /** Frequent Violence or Gore */
    FrequentViolenceOrGore = 1,
    /** Adult Only Sexual Content */
    AdultOnlySexualContent = 2,
    /** Frequent Nudity or Sexual Content */
    GratuitousSexualContent = 3,
    /** General Mature Content */
    AnyMatureContent = 4
}
```

Content descriptors let a creator flag mature content so Steam can filter the item according to each viewer's Mature Content preferences. Add and remove them in the same update; removals are applied before additions.

```ts
await client.workshop.updateItem(itemId, {
    contentDescriptors: [client.workshop.ContentDescriptor.FrequentViolenceOrGore],
    removeContentDescriptors: [client.workshop.ContentDescriptor.AnyMatureContent]
})
```

[`AddContentDescriptor`](https://partner.steamgames.com/doc/api/ISteamUGC#AddContentDescriptor) · [`RemoveContentDescriptor`](https://partner.steamgames.com/doc/api/ISteamUGC#RemoveContentDescriptor) · [`EUGCContentDescriptorID`](https://partner.steamgames.com/doc/api/ISteamUGC#EUGCContentDescriptorID)

#### `allowAdminTags`

```ts
allowAdminTags?: boolean   // default false
```

Only meaningful together with `tags`. Steam rejects a tag list containing admin-only tags unless this flag is set, and only accounts with the right permissions may set them — leave it `false` for player-facing publishing UI.

[`SetItemTags`](https://partner.steamgames.com/doc/api/ISteamUGC#SetItemTags)

### `updateItemWithCallback`

```ts
function updateItemWithCallback(
    itemId: bigint,
    updateDetails: UgcUpdate,
    appId: number | undefined | null,
    successCallback: (data: UgcResult) => void,
    errorCallback: (err: any) => void,
    progressCallback?: (data: UpdateProgress) => void,
    progressCallbackIntervalMs?: number | null
): void
```

The same update as `updateItem`, but callback-based and with progress reporting. Returns immediately; nothing to await.

Note that `appId` sits *before* the callbacks and is therefore positional — pass `null` to use the initialized app id.

```ts
interface UpdateProgress {
    status: UpdateStatus
    progress: bigint      // bytes done for the current status
    total: bigint         // bytes total for the current status
}

const enum UpdateStatus {
    Invalid = 0,
    PreparingConfig = 1,
    PreparingContent = 2,
    UploadingContent = 3,
    UploadingPreviewFile = 4,
    CommittingChanges = 5
}
```

`progress` and `total` are `bigint` byte counts, and they are **per status** — they reset when the status changes, so render the bar per phase rather than as one global percentage.

```ts
client.workshop.updateItemWithCallback(
    itemId,
    { contentPath: '/abs/path/to/content', changeNote: 'v2' },
    null,
    result => console.log('published', result.itemId),
    err => console.error('failed', err),
    p => {
        const pct = p.total > 0n ? Number((p.progress * 100n) / p.total) : 0
        console.log(`${client.workshop.UpdateStatus[p.status]} ${pct}%`)
    },
    250
)
```

Gotchas:

- `progressCallbackIntervalMs` defaults to **1000**. The polling happens on a background thread, so a small interval is cheap, but every tick crosses into JS.
- The progress loop stops as soon as it observes `Invalid` or `CommittingChanges`. `Invalid` also means "no update in flight", so a very fast update (or one whose first poll lands before Steam has started) can report a single `Invalid` tick and nothing else. Treat `successCallback`, not the progress stream, as the completion signal.
- `successCallback` and `errorCallback` are mutually exclusive; exactly one fires.

---

## Consuming

### `subscribe` / `unsubscribe`

```ts
function subscribe(itemId: bigint): Promise<void>
function unsubscribe(itemId: bigint): Promise<void>
```

`subscribe` adds the item to the user's subscriptions; Steam downloads and installs it as soon as it can. `unsubscribe` removes it — the files are only deleted **after the game quits**, so an unsubscribed item may still be installed for the rest of the session.

Both reject with Steam's error text on failure.

[`SubscribeItem`](https://partner.steamgames.com/doc/api/ISteamUGC#SubscribeItem) · [`UnsubscribeItem`](https://partner.steamgames.com/doc/api/ISteamUGC#UnsubscribeItem)

### `getSubscribedItems`

```ts
function getSubscribedItems(includeLocallyDisabled: boolean): Array<bigint>
```

The ids of every item the user is subscribed to for this app. Synchronous — it reads Steam's local state.

`includeLocallyDisabled` also returns items the user has disabled in the Steam client's workshop UI without unsubscribing. Pass `false` for "what should I actually load".

```ts
for (const id of client.workshop.getSubscribedItems(false)) {
    const info = client.workshop.installInfo(id)
    if (info) loadMod(info.folder)
}
```

### `state`

```ts
function state(itemId: bigint): number
```

The current local state of an item as an [`EItemState`](https://partner.steamgames.com/doc/api/ISteamUGC#EItemState) bitmask. States combine, so test with `&`:

| Bit | Meaning |
| --- | --- |
| `0` | Not tracked on this client |
| `1` | Subscribed |
| `2` | Legacy item (was published with the old workshop API) |
| `4` | Installed / on disk |
| `8` | Needs an update |
| `16` | Downloading right now |
| `32` | Download pending, will start when possible |

For example `9` is subscribed (`1`) plus needs an update (`8`).

```ts
const s = client.workshop.state(itemId)
const installed = (s & 4) !== 0
const needsUpdate = (s & 8) !== 0
const downloading = (s & 16) !== 0

if (installed && !needsUpdate) loadMod(client.workshop.installInfo(itemId)!.folder)
```

The library returns the raw number; there is no `EItemState` enum on the JS side, so define your own constants if you use this a lot.

[`GetItemState`](https://partner.steamgames.com/doc/api/ISteamUGC#GetItemState)

### `download` / `downloadInfo`

```ts
function download(itemId: bigint, highPriority: boolean): boolean
function downloadInfo(itemId: bigint): DownloadInfo | null
```

`download` starts (or resumes) a download of an item the user is subscribed to, returning `false` if the item id is invalid or the user is not logged in. `highPriority` pauses other in-progress Steam downloads and starts this one immediately — use it when the player is waiting on this specific item.

```ts
interface DownloadInfo {
    current: bigint    // bytes downloaded
    total: bigint      // bytes total
}
```

`downloadInfo` returns `null` when the item is not downloading (either finished or never started), so it is a poll, not an event. Combine it with `state` to tell "done" from "not started".

```ts
client.workshop.download(itemId, true)

const timer = setInterval(() => {
    const info = client.workshop.downloadInfo(itemId)
    if (!info) {
        clearInterval(timer)
        console.log('download finished or never started')
        return
    }
    console.log(`${info.current}/${info.total}`)
}, 500)
```

[`DownloadItem`](https://partner.steamgames.com/doc/api/ISteamUGC#DownloadItem) · [`GetItemDownloadInfo`](https://partner.steamgames.com/doc/api/ISteamUGC#GetItemDownloadInfo)

### `installInfo`

```ts
function installInfo(itemId: bigint): InstallInfo | null

interface InstallInfo {
    folder: string        // absolute path to the installed content
    sizeOnDisk: bigint
    timestamp: number     // unix epoch seconds of the last update
}
```

Returns `null` when the item is not installed on this machine. `folder` is the directory you load the mod from.

[`GetItemInstallInfo`](https://partner.steamgames.com/doc/api/ISteamUGC#GetItemInstallInfo)

### `suspendDownloads`

```ts
function suspendDownloads(suspend: boolean): void
```

Suspends or resumes **all** workshop downloads for the client. Useful during level loading or a cutscene, where Steam competing for disk and bandwidth costs you frame time.

```ts
client.workshop.suspendDownloads(true)
await loadLevel()
client.workshop.suspendDownloads(false)
```

Always pair the two calls — a suspend that is never resumed keeps the user's downloads stalled for the rest of the session.

[`SuspendDownloads`](https://partner.steamgames.com/doc/api/ISteamUGC#SuspendDownloads)

---

## Querying

All four query functions take the same optional `WorkshopItemQueryConfig` and return `WorkshopItem`s. Results may be served from Steam's cache (`wasCached`).

### `getItem`

```ts
function getItem(
    item: bigint,
    queryConfig?: WorkshopItemQueryConfig | null
): Promise<WorkshopItem | null>
```

Fetch a single item by id. Resolves to `null` when the query succeeded but returned nothing — a deleted, banned or nonexistent id — and rejects only when the query itself failed.

```ts
const item = await client.workshop.getItem(2748452276n, { includeMetadata: true })
if (item) console.log(item.title, item.owner.steamId64, item.metadata)
```

### `getItems`

```ts
function getItems(
    items: Array<bigint>,
    queryConfig?: WorkshopItemQueryConfig | null
): Promise<WorkshopItemsResult>

interface WorkshopItemsResult {
    items: Array<WorkshopItem | undefined | null>
    wasCached: boolean
}
```

Batch version of `getItem` — one round trip for many ids, which is what you want when hydrating a subscription list. Entries in `items` may be `null`/`undefined` for ids Steam could not resolve, so **always narrow before use**. Order is not guaranteed to match your input; match on `publishedFileId`.

```ts
const ids = client.workshop.getSubscribedItems(false)
const { items } = await client.workshop.getItems(ids, { includeKeyValueTags: true })

for (const item of items) {
    if (!item) continue
    console.log(item.publishedFileId, item.title)
}
```

### `getAllItems`

```ts
function getAllItems(
    page: number,
    queryType: UGCQueryType,
    itemType: UGCType,
    creatorAppId: number,
    consumerAppId: number,
    queryConfig?: WorkshopItemQueryConfig | null
): Promise<WorkshopPaginatedResult>

interface WorkshopPaginatedResult {
    items: Array<WorkshopItem | undefined | null>
    returnedResults: number
    totalResults: number
    wasCached: boolean
}
```

Browses the whole workshop for an app. `page` is handed straight to Steam, which numbers pages **from 1** and returns up to 50 items each; the page size is not configurable, so iterate until you have collected `totalResults` (or `returnedResults` comes back as 0).

`creatorAppId` is the app the item was *created* for and `consumerAppId` the app that *consumes* it; for a normal game both are your app id.

```ts
const perPage = 50
let page = 1
const all: bigint[] = []

for (;;) {
    const result = await client.workshop.getAllItems(
        page,
        client.workshop.UGCQueryType.RankedByTotalUniqueSubscriptions,
        client.workshop.UGCType.Items,
        480,
        480,
        { requiredTags: ['weapons'], matchAnyTag: false }
    )

    for (const item of result.items) if (item) all.push(item.publishedFileId)
    if (all.length >= result.totalResults || result.returnedResults === 0) break
    page++
}
```

[`CreateQueryAllUGCRequest`](https://partner.steamgames.com/doc/api/ISteamUGC#CreateQueryAllUGCRequest)

#### `UGCQueryType`

How the returned page is ranked.

```ts
const enum UGCQueryType {
    RankedByVote = 0,
    RankedByPublicationDate = 1,
    AcceptedForGameRankedByAcceptanceDate = 2,
    RankedByTrend = 3,
    FavoritedByFriendsRankedByPublicationDate = 4,
    CreatedByFriendsRankedByPublicationDate = 5,
    RankedByNumTimesReported = 6,
    CreatedByFollowedUsersRankedByPublicationDate = 7,
    NotYetRated = 8,
    RankedByTotalVotesAsc = 9,
    RankedByVotesUp = 10,
    RankedByTextSearch = 11,
    RankedByTotalUniqueSubscriptions = 12,
    RankedByPlaytimeTrend = 13,
    RankedByTotalPlaytime = 14,
    RankedByAveragePlaytimeTrend = 15,
    RankedByLifetimeAveragePlaytime = 16,
    RankedByPlaytimeSessionsTrend = 17,
    RankedByLifetimePlaytimeSessions = 18,
    RankedByLastUpdatedDate = 19
}
```

`RankedByTextSearch` is the one to pair with `searchText`. `RankedByTrend` respects `rankedByTrendDays`. The playtime rankings only produce anything if your game reports playtime for items.

#### `UGCType`

Which kind of UGC to return.

```ts
const enum UGCType {
    Items = 0,
    ItemsMtx = 1,
    ItemsReadyToUse = 2,
    Collections = 3,
    Artwork = 4,
    Videos = 5,
    Screenshots = 6,
    AllGuides = 7,
    WebGuides = 8,
    IntegratedGuides = 9,
    UsableInGame = 10,
    ControllerBindings = 11,
    GameManagedItems = 12,
    All = 13
}
```

`Items` is the normal choice for mods; `Collections` for browsing collections (pair it with `returnChildren`).

### `getUserItems`

```ts
function getUserItems(
    page: number,
    accountId: number,
    listType: UserListType,
    itemType: UGCType,
    sortOrder: UserListOrder,
    appIds: AppIDs,
    queryConfig?: WorkshopItemQueryConfig | null
): Promise<WorkshopPaginatedResult>

interface AppIDs {
    creator?: number
    consumer?: number
}
```

Queries one user's workshop lists — what they published, favorited, voted on, and so on.

`accountId` is the **32 bit account id**, not the SteamID64. Get it from `client.localplayer.getSteamId().accountId`, or from any `PlayerSteamId.accountId`.

Omitted `AppIDs` fields default to app id `0`; pass your app id for both in practice.

```ts
const me = client.localplayer.getSteamId()

const mine = await client.workshop.getUserItems(
    1,
    me.accountId,
    client.workshop.UserListType.Published,
    client.workshop.UGCType.Items,
    client.workshop.UserListOrder.LastUpdatedDesc,
    { creator: 480, consumer: 480 },
    { includeMetadata: true }
)

console.log(`${mine.totalResults} published items`)
```

```ts
const enum UserListType {
    Published = 0,
    VotedOn = 1,
    VotedUp = 2,
    VotedDown = 3,
    Favorited = 4,
    Subscribed = 5,
    UsedOrPlayed = 6,
    Followed = 7
}

const enum UserListOrder {
    CreationOrderAsc = 0,
    CreationOrderDesc = 1,
    TitleAsc = 2,
    LastUpdatedDesc = 3,
    SubscriptionDateDesc = 4,
    VoteScoreDesc = 5,
    ForModeration = 6
}
```

Most lists other than `Published` only work for the **local** user — Steam does not expose another user's subscriptions or votes.

[`CreateQueryUserUGCRequest`](https://partner.steamgames.com/doc/api/ISteamUGC#CreateQueryUserUGCRequest)

### `WorkshopItemQueryConfig`

```ts
interface WorkshopItemQueryConfig {
    cachedResponseMaxAge?: number
    includeMetadata?: boolean
    includeLongDescription?: boolean
    includeAdditionalPreviews?: boolean
    onlyIds?: boolean
    onlyTotal?: boolean
    language?: string
    matchAnyTag?: boolean
    requiredTags?: Array<string>
    excludedTags?: Array<string>
    searchText?: string
    rankedByTrendDays?: number
    returnChildren?: boolean
    includeKeyValueTags?: boolean
    cloudFileNameFilter?: string
    requiredKeyValueTags?: Array<KeyValueTag>
}
```

| Field | Effect |
| --- | --- |
| `cachedResponseMaxAge` | Accept a cached response up to this many seconds old; check `wasCached` on the result |
| `includeMetadata` | Populate `metadata` on each item |
| `includeLongDescription` | Return the full description instead of a truncated one |
| `includeAdditionalPreviews` | Ask Steam for the extra preview entries |
| `onlyIds` | Return only ids — cheapest query, most `WorkshopItem` fields come back empty |
| `onlyTotal` | Return only `totalResults`, no items at all |
| `language` | ISO language code for localized titles/descriptions |
| `matchAnyTag` | `true` = OR the `requiredTags`, `false` (default) = AND them |
| `requiredTags` | Tags an item must carry |
| `excludedTags` | Tags that disqualify an item |
| `searchText` | Full-text search; pair with `UGCQueryType.RankedByTextSearch` |
| `rankedByTrendDays` | Trend window in days for `RankedByTrend` |
| `returnChildren` | Populate `children` with the ids inside a collection |
| `includeKeyValueTags` | Populate `keyValueTags` on each item |
| `cloudFileNameFilter` | Only items whose cloud file name matches |
| `requiredKeyValueTags` | Key/value pairs an item must **all** carry |

Notes on the newer fields:

- **`includeKeyValueTags`** — without it `item.keyValueTags` is `undefined` even for items that have tags, because Steam simply does not send them. Same relationship as `includeMetadata` ↔ `item.metadata`.
- **`requiredKeyValueTags`** — every listed pair must be present on an item for it to be returned (AND, not OR; `matchAnyTag` does not apply here). This is the server-side way to fetch "all items in category `weapon`" without paging through everything.
- **`cloudFileNameFilter`** — matches on the item's cloud file name, useful when your items are identified by a fixed payload filename such as `level.dat`.
- **`returnChildren`** — only then is `children` populated. It costs an extra round trip inside Steam, so leave it off for plain item browsing.

```ts
const result = await client.workshop.getAllItems(
    1,
    client.workshop.UGCQueryType.RankedByPublicationDate,
    client.workshop.UGCType.Items,
    480,
    480,
    {
        includeKeyValueTags: true,
        includeMetadata: true,
        cloudFileNameFilter: 'level.dat',
        requiredKeyValueTags: [{ key: 'category', value: 'weapon' }]
    }
)
```

[`SetReturnKeyValueTags`](https://partner.steamgames.com/doc/api/ISteamUGC#SetReturnKeyValueTags) · [`AddRequiredKeyValueTag`](https://partner.steamgames.com/doc/api/ISteamUGC#AddRequiredKeyValueTag) · [`SetCloudFileNameFilter`](https://partner.steamgames.com/doc/api/ISteamUGC#SetCloudFileNameFilter) · [`SetReturnChildren`](https://partner.steamgames.com/doc/api/ISteamUGC#SetReturnChildren)

### `WorkshopItem`

```ts
interface WorkshopItem {
    publishedFileId: bigint
    creatorAppId?: number
    consumerAppId?: number
    title: string
    description: string
    owner: PlayerSteamId
    /** Time created in unix epoch seconds format */
    timeCreated: number
    /** Time updated in unix epoch seconds format */
    timeUpdated: number
    /** Time the user added it to their list, unix epoch seconds; not always applicable */
    timeAddedToUserList: number
    visibility: UgcItemVisibility
    banned: boolean
    acceptedForUse: boolean
    tags: Array<string>
    tagsTruncated: boolean
    url: string
    numUpvotes: number
    numDownvotes: number
    numChildren: number
    previewUrl?: string
    statistics: WorkshopItemStatistic
    children?: Array<bigint>
    keyValueTags?: Array<KeyValueTag>
    metadata?: string
}
```

- `owner` is a `PlayerSteamId` (`{ steamId64: bigint, steamId32: string, accountId: number }`).
- `tagsTruncated` is `true` when Steam had to cut the tag list short — `tags` is then incomplete.
- `previewUrl` is the workshop preview image; download it yourself, the library does not fetch it.
- `numChildren` is always present, but `children` is only filled when the query set `returnChildren`.
- `keyValueTags` is `undefined` both when the query did not ask for them and when the item has none — the two cases are indistinguishable from the result alone.
- `metadata` is `undefined` when the query did not ask for it, and also when the stored metadata is an empty string.

#### `WorkshopItemStatistic`

```ts
interface WorkshopItemStatistic {
    numSubscriptions?: bigint
    numFavorites?: bigint
    numFollowers?: bigint
    numUniqueSubscriptions?: bigint
    numUniqueFavorites?: bigint
    numUniqueFollowers?: bigint
    numUniqueWebsiteViews?: bigint
    reportScore?: bigint
    numSecondsPlayed?: bigint
    numPlaytimeSessions?: bigint
    numComments?: bigint
    numSecondsPlayedDuringTimePeriod?: bigint
    numPlaytimeSessionsDuringTimePeriod?: bigint
}
```

Every field is optional and a `bigint` — Steam returns each statistic independently, and any one of them may simply be absent for an item. The `...DuringTimePeriod` pair only means anything for trend queries. `statistics` itself is always present, even if every field inside it is `undefined`.

```ts
const subs = item.statistics.numUniqueSubscriptions ?? 0n
```

---

## Deleting

### `deleteItem`

```ts
function deleteItem(itemId: bigint): Promise<void>
```

> **This is permanent.** `deleteItem` deletes the workshop item from Steam outright — it is not an unpublish, not a visibility change, and there is no undo, no recycle bin, and no way to recover the id or its subscribers. Only the item's owner (or an app admin) can call it successfully.
>
> If you want to hide an item instead, set `visibility: UgcItemVisibility.Private` (or `Unlisted`) through `updateItem`.

Resolves once Steam confirms the deletion, rejects with Steam's error text otherwise. Put it behind an explicit, typed confirmation in any user-facing publishing tool.

```ts
if (await confirmDestructive(`Permanently delete "${item.title}"?`)) {
    await client.workshop.deleteItem(item.publishedFileId)
}
```

[`DeleteItem`](https://partner.steamgames.com/doc/api/ISteamUGC#DeleteItem)

---

## End-to-end example

Create an item, upload content with structured tags and metadata, then query it back and read them.

```ts
import * as steamworks from 'steamworks.js'

const APP_ID = 480
const client = steamworks.init(APP_ID)

async function publishLevel(contentDir: string, previewFile: string) {
    // 1. Reserve the item
    const created = await client.workshop.createItem(APP_ID, client.workshop.FileType.Community)
    const itemId: bigint = created.itemId

    if (created.needsToAcceptAgreement) {
        client.overlay.activateToWebPage(
            'https://steamcommunity.com/sharedfiles/workshoplegalagreement'
        )
    }

    // 2. Upload content, tags, key/value tags and metadata in one update
    await new Promise<void>((resolve, reject) => {
        client.workshop.updateItemWithCallback(
            itemId,
            {
                title: 'Crossfire Remastered',
                description: 'A remake of the classic arena.',
                changeNote: 'Initial release',
                contentPath: contentDir,          // absolute path to a folder
                previewPath: previewFile,         // absolute path to a jpg/png
                tags: ['maps', 'arena'],
                allowAdminTags: false,
                visibility: client.workshop.UgcItemVisibility.Public,
                metadata: JSON.stringify({ schema: 2, entry: 'level.dat' }),
                // Replace the whole key/value set: removals run before adds
                removeAllKeyValueTags: true,
                keyValueTags: [
                    { key: 'category', value: 'map' },
                    { key: 'players', value: '8' }
                ],
                contentDescriptors: [client.workshop.ContentDescriptor.FrequentViolenceOrGore]
            },
            null,                                  // appId: use the initialized one
            () => resolve(),
            err => reject(err),
            p => {
                const pct = p.total > 0n ? Number((p.progress * 100n) / p.total) : 0
                console.log(`${client.workshop.UpdateStatus[p.status]} ${pct}%`)
            },
            250
        )
    })

    // 3. Read it back, asking for the tags and metadata
    const item = await client.workshop.getItem(itemId, {
        includeKeyValueTags: true,
        includeMetadata: true,
        includeLongDescription: true
    })

    if (!item) throw new Error('item vanished right after publishing')

    console.log(item.title, item.url)
    console.log('tags:', item.tags.join(', '))
    for (const tag of item.keyValueTags ?? []) {
        console.log(`  ${tag.key} = ${tag.value}`)
    }

    const meta = item.metadata ? JSON.parse(item.metadata) : null
    console.log('schema', meta?.schema)
    console.log('subscribers', item.statistics.numUniqueSubscriptions ?? 0n)

    return itemId
}
```

And the consuming side in the game:

```ts
client.workshop.suspendDownloads(false)

for (const id of client.workshop.getSubscribedItems(false)) {
    const s = client.workshop.state(id)

    if ((s & 8) !== 0 || (s & 4) === 0) {
        client.workshop.download(id, false)   // needs update, or not installed yet
        continue
    }

    const info = client.workshop.installInfo(id)
    if (info) loadMod(info.folder)            // info.folder is an absolute path
}
```

---

## See also

- [[Getting-Started]] — `init()`, the BigInt rule, how promises are pumped
- [[API-Cloud]] — Steam Cloud storage, which `cloudFileNameFilter` matches against
- [[API-Overlay]] — sending the user to the workshop legal agreement or an item page
- [[API-Apps-Utils-and-LocalPlayer]] — `localplayer.getSteamId().accountId` for `getUserItems`
- [Steam Workshop implementation guide](https://partner.steamgames.com/doc/features/workshop/implementation)
