export declare function init(appId?: number | undefined | null): void
export declare function restartAppIfNecessary(appId: number): boolean
export declare function runCallbacks(): void
export interface PlayerSteamId {
  steamId64: bigint
  steamId32: string
  accountId: number
}
export declare namespace achievement {
  export function activate(achievement: string): boolean
  export function isActivated(achievement: string): boolean
  export function clear(achievement: string): boolean
  export function names(): Array<string>
}
export declare namespace apps {
  export function isSubscribedApp(appId: number): boolean
  export function isAppInstalled(appId: number): boolean
  export function isDlcInstalled(appId: number): boolean
  export function isSubscribedFromFreeWeekend(): boolean
  export function isVacBanned(): boolean
  export function isCybercafe(): boolean
  export function isLowViolence(): boolean
  export function isSubscribed(): boolean
  export function appBuildId(): number
  export function appInstallDir(appId: number): string
  export function appOwner(): PlayerSteamId
  export function availableGameLanguages(): Array<string>
  export function currentGameLanguage(): string
  export function currentBetaName(): string | null
}
export declare namespace auth {
  /**
   * @param steamId64 - The user steam id or game server steam id. Use as NetworkIdentity of the remote system that will authenticate the ticket. If it is peer-to-peer then the user steam ID. If it is a game server, then the game server steam ID may be used if it was obtained from a trusted 3rd party
   * @param timeoutSeconds - The number of seconds to wait for the ticket to be validated. Default value is 10 seconds.
   */
  export function getSessionTicketWithSteamId(steamId64: bigint, timeoutSeconds?: number | undefined | null): Promise<Ticket>
  /**
   * @param ip - The string of IPv4 or IPv6 address. Use as NetworkIdentity of the remote system that will authenticate the ticket.
   * @param timeoutSeconds - The number of seconds to wait for the ticket to be validated. Default value is 10 seconds.
   */
  export function getSessionTicketWithIp(ip: string, timeoutSeconds?: number | undefined | null): Promise<Ticket>
  export function getAuthTicketForWebApi(identity: string, timeoutSeconds?: number | undefined | null): Promise<Ticket>
  export class Ticket {
    cancel(): void
    getBytes(): Buffer
  }
}
export declare namespace callback {
  export const enum SteamCallback {
    PersonaStateChange = 0,
    SteamServersConnected = 1,
    SteamServersDisconnected = 2,
    SteamServerConnectFailure = 3,
    LobbyDataUpdate = 4,
    LobbyChatUpdate = 5,
    P2PSessionRequest = 6,
    P2PSessionConnectFail = 7,
    GameLobbyJoinRequested = 8,
    MicroTxnAuthorizationResponse = 9
  }
  export function register<C extends keyof import('./callbacks').CallbackReturns>(steamCallback: C, handler: (value: import('./callbacks').CallbackReturns[C]) => void): Handle
  export class Handle {
    disconnect(): void
  }
}
export declare namespace cloud {
  export function isEnabledForAccount(): boolean
  export function isEnabledForApp(): boolean
  export function setEnabledForApp(enabled: boolean): void
  export function readFile(name: string): string
  export function writeFile(name: string, content: string): boolean
  export function deleteFile(name: string): boolean
  export function fileExists(name: string): boolean
  export function isFilePersisted(name: string): boolean
  export function fileTimestamp(name: string): number
  export function listFiles(): Array<FileInfo>
  export class FileInfo {
    name: string
    size: bigint
  }
}
export declare namespace friends {
  /**
   * Flags to filter the friends list by relationship.
   * See [Steam API](https://partner.steamgames.com/doc/api/ISteamFriends#EFriendFlags)
   */
  export const enum FriendFlags {
    None = 0,
    Blocked = 1,
    FriendshipRequested = 2,
    /** The usual friends list. */
    Immediate = 4,
    ClanMember = 8,
    OnGameServer = 16,
    RequestingFriendship = 128,
    RequestingInfo = 256,
    Ignored = 512,
    IgnoredFriend = 1024,
    ChatMember = 4096,
    All = 65535
  }
  /**
   * The online state of a user.
   * See [Steam API](https://partner.steamgames.com/doc/api/ISteamFriends#EPersonaState)
   */
  export const enum FriendState {
    Offline = 0,
    Online = 1,
    Busy = 2,
    Away = 3,
    Snooze = 4,
    LookingToTrade = 5,
    LookingToPlay = 6
  }
  /**
   * Information about the game a user is currently playing.
   * See [Steam API](https://partner.steamgames.com/doc/api/ISteamFriends#GetFriendGamePlayed)
   */
  export interface FriendGame {
    /** The id of the game being played. */
    gameId: bigint
    /** The app id of the game being played. */
    appId: number
    /** The IPv4 address of the server the player is on, "0.0.0.0" if none. */
    gameAddress: string
    /** The game port of the server the player is on, 0 if none. */
    gamePort: number
    /** The query port of the server the player is on, 0 if none. */
    queryPort: number
    /** The id of the lobby the player is in, 0 if none. */
    lobbyId: bigint
  }
  /**
   * Get the users matching the given relationship, the regular friends list by default.
   * See [Steam API](https://partner.steamgames.com/doc/api/ISteamFriends#GetFriendByIndex)
   */
  export function getFriends(flags?: FriendFlags | undefined | null): Array<Friend>
  /**
   * Get the users on the local user's recently-played-with list.
   * See [Steam API](https://partner.steamgames.com/doc/api/ISteamFriends#GetCoplayFriend)
   */
  export function getCoplayFriends(): Array<Friend>
  /** Get an arbitrary user by steam id, they don't have to be a friend. */
  export function getFriend(steamId64: bigint): Friend
  /**
   * Request the persona name and optionally the avatar of a user from Steam.
   * @param nameOnly - Only request the name, skipping the avatar.
   * @returns true if the information is being fetched, false if it was already available.
   * See [Steam API](https://partner.steamgames.com/doc/api/ISteamFriends#RequestUserInformation)
   */
  export function requestUserInformation(steamId64: bigint, nameOnly: boolean): boolean
  /**
   * A Steam user, as seen through the friends interface.
   * See [Steam API](https://partner.steamgames.com/doc/api/ISteamFriends)
   */
  export class Friend {
    /** The steam id of this user. */
    getSteamId(): PlayerSteamId
    /** The current display (persona) name of this user. */
    getName(): string
    /** The nickname the local user has set for this user, if any. */
    getNickName(): string | null
    /** The online state of this user. */
    getState(): FriendState
    /** Information about the game this user is currently playing, if any. */
    getGamePlayed(): FriendGame | null
    /** Whether this user matches the given relationship criteria. */
    hasFriend(flags: FriendFlags): boolean
    /**
     * The small avatar of this user as raw RGBA bytes, 32x32 pixels (4096 bytes).
     * Returns null when the avatar is not loaded yet, use `requestUserInformation` to fetch it.
     */
    smallAvatar(): Buffer | null
    /**
     * The medium avatar of this user as raw RGBA bytes, 64x64 pixels (16384 bytes).
     * Returns null when the avatar is not loaded yet, use `requestUserInformation` to fetch it.
     */
    mediumAvatar(): Buffer | null
    /**
     * The large avatar of this user as raw RGBA bytes, 184x184 pixels (135424 bytes).
     * Returns null when the avatar is not loaded yet, use `requestUserInformation` to fetch it.
     */
    largeAvatar(): Buffer | null
  }
}
export declare namespace input {
  export const enum InputType {
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
  export interface AnalogActionVector {
    x: number
    y: number
  }
  export function init(): void
  export function getControllers(): Array<Controller>
  export function getActionSet(actionSetName: string): bigint
  export function getDigitalAction(actionName: string): bigint
  export function getAnalogAction(actionName: string): bigint
  export function shutdown(): void
  export class Controller {
    activateActionSet(actionSetHandle: bigint): void
    isDigitalActionPressed(actionHandle: bigint): boolean
    getAnalogActionVector(actionHandle: bigint): AnalogActionVector
    getType(): InputType
    getHandle(): bigint
  }
}
export declare namespace leaderboard {
  /**
   * The sort order of a leaderboard.
   * {@link https://partner.steamgames.com/doc/api/ISteamUserStats#ELeaderboardSortMethod}
   */
  export const enum LeaderboardSortMethod {
    /** The top-score is the lowest number. */
    Ascending = 0,
    /** The top-score is the highest number. */
    Descending = 1
  }
  /**
   * How a leaderboard score is displayed in the Steam overlay and community.
   * {@link https://partner.steamgames.com/doc/api/ISteamUserStats#ELeaderboardDisplayType}
   */
  export const enum LeaderboardDisplayType {
    /** The score is just a simple numerical value. */
    Numeric = 0,
    /** The score represents a time, in seconds. */
    TimeSeconds = 1,
    /** The score represents a time, in milliseconds. */
    TimeMilliSeconds = 2
  }
  /**
   * How an uploaded score is treated when the user already has an entry.
   * {@link https://partner.steamgames.com/doc/api/ISteamUserStats#ELeaderboardUploadScoreMethod}
   */
  export const enum LeaderboardUploadScoreMethod {
    /** Only replaces the existing entry if the new score is better. */
    KeepBest = 0,
    /** Always replaces the existing entry, even with a worse score. */
    ForceUpdate = 1
  }
  /**
   * Which set of leaderboard entries to download.
   * {@link https://partner.steamgames.com/doc/api/ISteamUserStats#ELeaderboardDataRequest}
   */
  export const enum LeaderboardDataRequest {
    /** Query everyone on the leaderboard, `start` and `end` are absolute ranks (1 based). */
    Global = 0,
    /** Query around the current user, `start` and `end` are relative to the user's rank. */
    GlobalAroundUser = 1,
    /** Query the current user's friends, `start` and `end` are ignored. */
    Friends = 2
  }
  /**
   * The outcome of a successful score upload.
   * {@link https://partner.steamgames.com/doc/api/ISteamUserStats#LeaderboardScoreUploaded_t}
   */
  export interface LeaderboardScoreUploaded {
    /** The score that was submitted. */
    score: number
    /**
     * Whether the score on the leaderboard actually changed.
     * False when `KeepBest` was used and the existing score was better.
     */
    scoreChanged: boolean
    /** The new global rank of the user, 0 when the score did not change. */
    globalRankNew: number
    /** The global rank the user had before this upload, 0 when they had no entry. */
    globalRankPrevious: number
  }
  /**
   * A single downloaded leaderboard entry.
   * {@link https://partner.steamgames.com/doc/api/ISteamUserStats#LeaderboardEntry_t}
   */
  export interface LeaderboardEntry {
    /** The user that owns this entry. */
    steamId: PlayerSteamId
    /** The global rank of this entry, 1 based. */
    globalRank: number
    /** The score of this entry. */
    score: number
    /**
     * The game specific details uploaded with the score.
     * Empty unless `maxDetailsLen` was greater than 0 when downloading.
     */
    details: Array<number>
  }
  /**
   * Find a leaderboard by its name, as configured on the Steamworks partner site.
   *
   * Resolves to null when no leaderboard with that name exists.
   * {@link https://partner.steamgames.com/doc/api/ISteamUserStats#FindLeaderboard}
   */
  export function findLeaderboard(name: string): Promise<Leaderboard | null>
  /**
   * Find a leaderboard by name, creating it if it does not exist yet.
   *
   * The sort method and display type are only used when the leaderboard is created; an
   * existing leaderboard keeps the settings it was created with.
   * {@link https://partner.steamgames.com/doc/api/ISteamUserStats#FindOrCreateLeaderboard}
   */
  export function findOrCreateLeaderboard(name: string, sortMethod: LeaderboardSortMethod, displayType: LeaderboardDisplayType): Promise<Leaderboard>
  /**
   * A handle to a Steam leaderboard, obtained through {@link findLeaderboard} or
   * {@link findOrCreateLeaderboard}.
   * {@link https://partner.steamgames.com/doc/api/ISteamUserStats}
   */
  export class Leaderboard {
    /** The raw `SteamLeaderboard_t` handle. */
    handle: bigint
    /**
     * Get the name of this leaderboard.
     * {@link https://partner.steamgames.com/doc/api/ISteamUserStats#GetLeaderboardName}
     */
    getName(): string
    /**
     * Get the total number of entries in this leaderboard.
     * {@link https://partner.steamgames.com/doc/api/ISteamUserStats#GetLeaderboardEntryCount}
     */
    getEntryCount(): number
    /**
     * Get the sort method of this leaderboard, or null if the handle is invalid.
     * {@link https://partner.steamgames.com/doc/api/ISteamUserStats#GetLeaderboardSortMethod}
     */
    getSortMethod(): LeaderboardSortMethod | null
    /**
     * Get the display type of this leaderboard, or null if the handle is invalid.
     * {@link https://partner.steamgames.com/doc/api/ISteamUserStats#GetLeaderboardDisplayType}
     */
    getDisplayType(): LeaderboardDisplayType | null
    /**
     * Upload a score to this leaderboard for the current user.
     *
     * `details` is optional game specific data (at most 64 entries) stored alongside the
     * score, for example a replay seed or a per-level breakdown.
     * {@link https://partner.steamgames.com/doc/api/ISteamUserStats#UploadLeaderboardScore}
     */
    uploadScore(score: number, method: LeaderboardUploadScoreMethod, details?: Array<number> | undefined | null): Promise<LeaderboardScoreUploaded>
    /**
     * Download a range of entries from this leaderboard.
     *
     * The meaning of `start` and `end` depends on `request`: absolute 1 based ranks for
     * `Global`, offsets relative to the current user for `GlobalAroundUser` (where `start`
     * is usually negative, e.g. -4 to 5 for the ten entries around the user), and ignored
     * for `Friends`. `maxDetailsLen` is how many details entries to read per row (0 to 64),
     * pass 0 when the leaderboard has no details.
     * {@link https://partner.steamgames.com/doc/api/ISteamUserStats#DownloadLeaderboardEntries}
     */
    downloadEntries(request: LeaderboardDataRequest, start: number, end: number, maxDetailsLen: number): Promise<Array<LeaderboardEntry>>
  }
}
export declare namespace localplayer {
  export function getSteamId(): PlayerSteamId
  export function getName(): string
  export function getLevel(): number
  /** @returns the 2 digit ISO 3166-1-alpha-2 format country code which client is running in, e.g. "US" or "UK". */
  export function getIpCountry(): string
  export function setRichPresence(key: string, value?: string | undefined | null): void
}
export declare namespace matchmaking {
  export const enum LobbyType {
    Private = 0,
    FriendsOnly = 1,
    Public = 2,
    Invisible = 3
  }
  /**
   * Comparison used by a string lobby list filter.
   * @see https://partner.steamgames.com/doc/api/ISteamMatchmaking#ELobbyComparison
   */
  export const enum LobbyStringComparison {
    EqualToOrLessThan = 0,
    LessThan = 1,
    Equal = 2,
    GreaterThan = 3,
    EqualToOrGreaterThan = 4,
    NotEqual = 5
  }
  /**
   * Comparison used by a numerical lobby list filter.
   * @see https://partner.steamgames.com/doc/api/ISteamMatchmaking#ELobbyComparison
   */
  export const enum LobbyNumberComparison {
    Equal = 0,
    NotEqual = 1,
    GreaterThan = 2,
    GreaterThanEqualTo = 3,
    LessThan = 4,
    LessThanEqualTo = 5
  }
  /**
   * How far geographically the returned lobbies may be.
   * @see https://partner.steamgames.com/doc/api/ISteamMatchmaking#ELobbyDistanceFilter
   */
  export const enum LobbyDistanceFilter {
    Close = 0,
    Default = 1,
    Far = 2,
    Worldwide = 3
  }
  /**
   * Matches lobbies whose string metadata compares against `value` as requested.
   * @see https://partner.steamgames.com/doc/api/ISteamMatchmaking#AddRequestLobbyListStringFilter
   */
  export interface LobbyStringFilter {
    key: string
    value: string
    comparison: LobbyStringComparison
  }
  /**
   * Matches lobbies whose numerical metadata compares against `value` as requested.
   * @see https://partner.steamgames.com/doc/api/ISteamMatchmaking#AddRequestLobbyListNumericalFilter
   */
  export interface LobbyNumberFilter {
    key: string
    value: number
    comparison: LobbyNumberComparison
  }
  /**
   * Sorts the results by how close their metadata is to `value`. This does not filter anything out.
   * @see https://partner.steamgames.com/doc/api/ISteamMatchmaking#AddRequestLobbyListNearValueFilter
   */
  export interface LobbyNearFilter {
    key: string
    value: number
  }
  /**
   * Filters applied to a lobby list request. Every field is optional; an empty
   * filter returns the same lobbies as an unfiltered request.
   * @see https://partner.steamgames.com/doc/api/ISteamMatchmaking#RequestLobbyList
   */
  export interface LobbyListFilter {
    /** String metadata comparisons a lobby has to satisfy */
    stringFilters?: Array<LobbyStringFilter>
    /** Numerical metadata comparisons a lobby has to satisfy */
    numberFilters?: Array<LobbyNumberFilter>
    /** Metadata values the results are sorted closest to */
    nearValueFilters?: Array<LobbyNearFilter>
    /** Only return lobbies with at least this many open slots (0-255) */
    slotsAvailable?: number
    /** How far geographically the returned lobbies may be */
    distance?: LobbyDistanceFilter
    /** Maximum amount of lobbies to return */
    count?: number
  }
  export function createLobby(lobbyType: LobbyType, maxMembers: number): Promise<Lobby>
  export function joinLobby(lobbyId: bigint): Promise<Lobby>
  /**
   * Get the list of lobbies for this app, optionally narrowed down by `filter`.
   * Calling this without a filter returns the unfiltered lobby list.
   * @see https://partner.steamgames.com/doc/api/ISteamMatchmaking#RequestLobbyList
   */
  export function getLobbies(filter?: LobbyListFilter | undefined | null): Promise<Array<Lobby>>
  export class Lobby {
    id: bigint
    join(): Promise<Lobby>
    leave(): void
    openInviteDialog(): void
    getMemberCount(): bigint
    getMemberLimit(): bigint | null
    getMembers(): Array<PlayerSteamId>
    getOwner(): PlayerSteamId
    setJoinable(joinable: boolean): boolean
    getData(key: string): string | null
    setData(key: string, value: string): boolean
    deleteData(key: string): boolean
    /** Get an object containing all the lobby data */
    getFullData(): Record<string, string>
    /**
     * Merge current lobby data with provided data in a single batch
     * @returns true if all data was set successfully
     */
    mergeFullData(data: Record<string, string>): boolean
  }
}
export declare namespace networking {
  export interface P2PPacket {
    data: Buffer
    size: number
    steamId: PlayerSteamId
  }
  /** The method used to send a packet */
  export const enum SendType {
    /**
     * Send the packet directly over udp.
     *
     * Can't be larger than 1200 bytes
     */
    Unreliable = 0,
    /**
     * Like `Unreliable` but doesn't buffer packets
     * sent before the connection has started.
     */
    UnreliableNoDelay = 1,
    /**
     * Reliable packet sending.
     *
     * Can't be larger than 1 megabyte.
     */
    Reliable = 2,
    /**
     * Like `Reliable` but applies the nagle
     * algorithm to packets being sent
     */
    ReliableWithBuffering = 3
  }
  export function sendP2PPacket(steamId64: bigint, sendType: SendType, data: Buffer): boolean
  export function isP2PPacketAvailable(): number
  export function readP2PPacket(size: number): P2PPacket
  export function acceptP2PSession(steamId64: bigint): void
}
export declare namespace overlay {
  export const enum Dialog {
    Friends = 0,
    Community = 1,
    Players = 2,
    Settings = 3,
    OfficialGameGroup = 4,
    Stats = 5,
    Achievements = 6
  }
  export const enum StoreFlag {
    None = 0,
    AddToCart = 1,
    AddToCartAndShow = 2
  }
  export function activateDialog(dialog: Dialog): void
  export function activateDialogToUser(dialog: Dialog, steamId64: bigint): void
  export function activateInviteDialog(lobbyId: bigint): void
  export function activateToWebPage(url: string): void
  export function activateToStore(appId: number, flag: StoreFlag): void
}
export declare namespace stats {
  export function getInt(name: string): number | null
  export function setInt(name: string, value: number): boolean
  export function getFloat(name: string): number | null
  export function setFloat(name: string, value: number): boolean
  export function store(): boolean
  export function resetAll(achievementsToo: boolean): boolean
}
export declare namespace utils {
  export function getAppId(): number
  export function getServerRealTime(): number
  export function isSteamRunningOnSteamDeck(): boolean
  export const enum GamepadTextInputMode {
    Normal = 0,
    Password = 1
  }
  export const enum GamepadTextInputLineMode {
    SingleLine = 0,
    MultipleLines = 1
  }
  /** @returns the entered text, or null if cancelled or could not show the input */
  export function showGamepadTextInput(inputMode: GamepadTextInputMode, inputLineMode: GamepadTextInputLineMode, description: string, maxCharacters: number, existingText?: string | undefined | null): Promise<string | null>
  export const enum FloatingGamepadTextInputMode {
    SingleLine = 0,
    MultipleLines = 1,
    Email = 2,
    Numeric = 3
  }
  /** @returns true if the floating keyboard was shown, otherwise, false */
  export function showFloatingGamepadTextInput(keyboardMode: FloatingGamepadTextInputMode, x: number, y: number, width: number, height: number): Promise<boolean>
}
export declare namespace workshop {
  export interface UgcResult {
    itemId: bigint
    needsToAcceptAgreement: boolean
  }
  export const enum UgcItemVisibility {
    Public = 0,
    FriendsOnly = 1,
    Private = 2,
    Unlisted = 3
  }
  export interface UgcUpdate {
    title?: string
    description?: string
    changeNote?: string
    previewPath?: string
    contentPath?: string
    tags?: Array<string>
    visibility?: UgcItemVisibility
  }
  export interface InstallInfo {
    folder: string
    sizeOnDisk: bigint
    timestamp: number
  }
  export interface DownloadInfo {
    current: bigint
    total: bigint
  }
  export const enum UpdateStatus {
    Invalid = 0,
    PreparingConfig = 1,
    PreparingContent = 2,
    UploadingContent = 3,
    UploadingPreviewFile = 4,
    CommittingChanges = 5
  }
  export interface UpdateProgress {
    status: UpdateStatus
    progress: bigint
    total: bigint
  }
  export const enum FileType {
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
  /** Create a new workshop item. `fileType` defaults to `Community` when not provided. */
  export function createItem(appId?: number | undefined | null, fileType?: FileType | undefined | null): Promise<UgcResult>
  export function updateItem(itemId: bigint, updateDetails: UgcUpdate, appId?: number | undefined | null): Promise<UgcResult>
  export function updateItemWithCallback(itemId: bigint, updateDetails: UgcUpdate, appId: number | undefined | null, successCallback: (data: UgcResult) => void, errorCallback: (err: any) => void, progressCallback?: (data: UpdateProgress) => void, progressCallbackIntervalMs?: number | undefined | null): void
  /**
   * Subscribe to a workshop item. It will be downloaded and installed as soon as possible.
   *
   * {@link https://partner.steamgames.com/doc/api/ISteamUGC#SubscribeItem}
   */
  export function subscribe(itemId: bigint): Promise<void>
  /**
   * Unsubscribe from a workshop item. This will result in the item being removed after the game quits.
   *
   * {@link https://partner.steamgames.com/doc/api/ISteamUGC#UnsubscribeItem}
   */
  export function unsubscribe(itemId: bigint): Promise<void>
  /**
   * Gets the current state of a workshop item on this client. States can be combined.
   *
   * @returns a number with the current item state, e.g. 9
   * 9 = 1 (The current user is subscribed to this item) + 8 (The item needs an update)
   *
   * {@link https://partner.steamgames.com/doc/api/ISteamUGC#GetItemState}
   * {@link https://partner.steamgames.com/doc/api/ISteamUGC#EItemState}
   */
  export function state(itemId: bigint): number
  /**
   * Gets info about currently installed content on the disc for workshop item.
   *
   * @returns an object with the the properties {folder, size_on_disk, timestamp}
   *
   * {@link https://partner.steamgames.com/doc/api/ISteamUGC#GetItemInstallInfo}
   */
  export function installInfo(itemId: bigint): InstallInfo | null
  /**
   * Get info about a pending download of a workshop item.
   *
   * @returns an object with the properties {current, total}
   *
   * {@link https://partner.steamgames.com/doc/api/ISteamUGC#GetItemDownloadInfo}
   */
  export function downloadInfo(itemId: bigint): DownloadInfo | null
  /**
   * Download or update a workshop item.
   *
   * @param highPriority - If high priority is true, start the download in high priority mode, pausing any existing in-progress Steam downloads and immediately begin downloading this workshop item.
   * @returns true or false
   *
   * {@link https://partner.steamgames.com/doc/api/ISteamUGC#DownloadItem}
   */
  export function download(itemId: bigint, highPriority: boolean): boolean
  /**
   * Get all subscribed workshop items.
   * @returns an array of subscribed workshop item ids
   */
  export function getSubscribedItems(includeLocallyDisabled: boolean): Array<bigint>
  export function deleteItem(itemId: bigint): Promise<void>
  export const enum UGCQueryType {
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
  export const enum UGCType {
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
  export const enum UserListType {
    Published = 0,
    VotedOn = 1,
    VotedUp = 2,
    VotedDown = 3,
    Favorited = 4,
    Subscribed = 5,
    UsedOrPlayed = 6,
    Followed = 7
  }
  export const enum UserListOrder {
    CreationOrderAsc = 0,
    CreationOrderDesc = 1,
    TitleAsc = 2,
    LastUpdatedDesc = 3,
    SubscriptionDateDesc = 4,
    VoteScoreDesc = 5,
    ForModeration = 6
  }
  export interface WorkshopItemStatistic {
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
  export interface WorkshopItem {
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
    /** Time when the user added the published item to their list (not always applicable), provided in Unix epoch format (time since Jan 1st, 1970). */
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
  }
  export interface WorkshopPaginatedResult {
    items: Array<WorkshopItem | undefined | null>
    returnedResults: number
    totalResults: number
    wasCached: boolean
  }
  export interface WorkshopItemsResult {
    items: Array<WorkshopItem | undefined | null>
    wasCached: boolean
  }
  export interface WorkshopItemQueryConfig {
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
  }
  export interface AppIDs {
    creator?: number
    consumer?: number
  }
  export function getItem(item: bigint, queryConfig?: WorkshopItemQueryConfig | undefined | null): Promise<WorkshopItem | null>
  export function getItems(items: Array<bigint>, queryConfig?: WorkshopItemQueryConfig | undefined | null): Promise<WorkshopItemsResult>
  export function getAllItems(page: number, queryType: UGCQueryType, itemType: UGCType, creatorAppId: number, consumerAppId: number, queryConfig?: WorkshopItemQueryConfig | undefined | null): Promise<WorkshopPaginatedResult>
  export function getUserItems(page: number, accountId: number, listType: UserListType, itemType: UGCType, sortOrder: UserListOrder, appIds: AppIDs, queryConfig?: WorkshopItemQueryConfig | undefined | null): Promise<WorkshopPaginatedResult>
}
