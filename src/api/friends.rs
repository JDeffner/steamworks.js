use napi_derive::napi;

#[napi]
pub mod friends {
    use crate::api::localplayer::PlayerSteamId;
    use napi::bindgen_prelude::{BigInt, Buffer};
    use steamworks::SteamId;

    /// Flags to filter the friends list by relationship.
    /// See [Steam API](https://partner.steamgames.com/doc/api/ISteamFriends#EFriendFlags)
    #[napi]
    pub enum FriendFlags {
        None = 0x0000,
        Blocked = 0x0001,
        FriendshipRequested = 0x0002,
        /// The usual friends list.
        Immediate = 0x0004,
        ClanMember = 0x0008,
        OnGameServer = 0x0010,
        RequestingFriendship = 0x0080,
        RequestingInfo = 0x0100,
        Ignored = 0x0200,
        IgnoredFriend = 0x0400,
        ChatMember = 0x1000,
        All = 0xFFFF,
    }

    // Flag parameters are plain u32 bitmasks so callers can OR variants together the way
    // EFriendFlags supports; unknown bits are dropped rather than rejected.
    fn friend_flags_from_bits(flags: u32) -> steamworks::FriendFlags {
        steamworks::FriendFlags::from_bits_truncate(flags as u16)
    }

    /// The online state of a user.
    /// See [Steam API](https://partner.steamgames.com/doc/api/ISteamFriends#EPersonaState)
    #[napi]
    pub enum FriendState {
        Offline,
        Online,
        Busy,
        Away,
        Snooze,
        LookingToTrade,
        LookingToPlay,
        Invisible,
    }

    /// Information about the game a user is currently playing.
    /// See [Steam API](https://partner.steamgames.com/doc/api/ISteamFriends#GetFriendGamePlayed)
    #[napi(object)]
    pub struct FriendGame {
        /// The id of the game being played.
        pub game_id: BigInt,
        /// The app id of the game being played.
        pub app_id: u32,
        /// The IPv4 address of the server the player is on, "0.0.0.0" if none.
        pub game_address: String,
        /// The game port of the server the player is on, 0 if none.
        pub game_port: u16,
        /// The query port of the server the player is on, 0 if none.
        pub query_port: u16,
        /// The id of the lobby the player is in, 0 if none.
        pub lobby_id: BigInt,
    }

    impl From<steamworks::FriendGame> for FriendGame {
        fn from(game: steamworks::FriendGame) -> Self {
            Self {
                game_id: BigInt::from(game.game.raw()),
                app_id: game.game.app_id().0,
                game_address: game.game_address.to_string(),
                game_port: game.game_port,
                query_port: game.query_port,
                lobby_id: BigInt::from(game.lobby.raw()),
            }
        }
    }

    /// A Steam user, as seen through the friends interface.
    /// See [Steam API](https://partner.steamgames.com/doc/api/ISteamFriends)
    #[napi]
    pub struct Friend {
        steam_id: SteamId,
    }

    impl Friend {
        pub(crate) fn from_steamid(steam_id: SteamId) -> Self {
            Self { steam_id }
        }
    }

    #[napi]
    impl Friend {
        /// The steam id of this user.
        #[napi]
        pub fn get_steam_id(&self) -> PlayerSteamId {
            PlayerSteamId::from_steamid(self.steam_id)
        }

        /// The current display (persona) name of this user.
        #[napi]
        pub fn get_name(&self) -> String {
            let client = crate::client::get_client();
            client.friends().get_friend(self.steam_id).name()
        }

        /// The nickname the local user has set for this user, if any.
        #[napi]
        pub fn get_nick_name(&self) -> Option<String> {
            let client = crate::client::get_client();
            client.friends().get_friend(self.steam_id).nick_name()
        }

        /// The online state of this user.
        #[napi]
        pub fn get_state(&self) -> FriendState {
            // Hold the client to guarantee Steam is initialized before touching the raw
            // interface. The raw call sidesteps steamworks-rs' Friend::state(), whose
            // non-exhaustive match panics on k_EPersonaStateInvisible and would abort the
            // whole process from a sync napi method.
            let _client = crate::client::get_client();
            let state = unsafe {
                let friends = steamworks::sys::SteamAPI_SteamFriends_v018();
                steamworks::sys::SteamAPI_ISteamFriends_GetFriendPersonaState(
                    friends,
                    self.steam_id.raw(),
                )
            };
            use steamworks::sys::EPersonaState;
            match state {
                EPersonaState::k_EPersonaStateOnline => FriendState::Online,
                EPersonaState::k_EPersonaStateBusy => FriendState::Busy,
                EPersonaState::k_EPersonaStateAway => FriendState::Away,
                EPersonaState::k_EPersonaStateSnooze => FriendState::Snooze,
                EPersonaState::k_EPersonaStateLookingToTrade => FriendState::LookingToTrade,
                EPersonaState::k_EPersonaStateLookingToPlay => FriendState::LookingToPlay,
                EPersonaState::k_EPersonaStateInvisible => FriendState::Invisible,
                // Offline, plus any state added by a future SDK.
                _ => FriendState::Offline,
            }
        }

        /// Information about the game this user is currently playing, if any.
        #[napi]
        pub fn get_game_played(&self) -> Option<FriendGame> {
            let client = crate::client::get_client();
            client
                .friends()
                .get_friend(self.steam_id)
                .game_played()
                .map(FriendGame::from)
        }

        /// Whether this user matches the given relationship criteria.
        /// `flags` is a bitmask of `FriendFlags` values, which may be OR-ed together.
        #[napi]
        pub fn has_friend(&self, flags: u32) -> bool {
            let client = crate::client::get_client();
            client
                .friends()
                .get_friend(self.steam_id)
                .has_friend(friend_flags_from_bits(flags))
        }

        /// The small avatar of this user as raw RGBA bytes, 32x32 pixels (4096 bytes).
        /// Returns null when the avatar is not loaded yet, use `requestUserInformation` to fetch it.
        #[napi]
        pub fn small_avatar(&self) -> Option<Buffer> {
            let client = crate::client::get_client();
            client
                .friends()
                .get_friend(self.steam_id)
                .small_avatar()
                .map(Buffer::from)
        }

        /// The medium avatar of this user as raw RGBA bytes, 64x64 pixels (16384 bytes).
        /// Returns null when the avatar is not loaded yet, use `requestUserInformation` to fetch it.
        #[napi]
        pub fn medium_avatar(&self) -> Option<Buffer> {
            let client = crate::client::get_client();
            client
                .friends()
                .get_friend(self.steam_id)
                .medium_avatar()
                .map(Buffer::from)
        }

        /// The large avatar of this user as raw RGBA bytes, 184x184 pixels (135424 bytes).
        /// Returns null when the avatar is not loaded yet, use `requestUserInformation` to fetch it.
        #[napi]
        pub fn large_avatar(&self) -> Option<Buffer> {
            let client = crate::client::get_client();
            client
                .friends()
                .get_friend(self.steam_id)
                .large_avatar()
                .map(Buffer::from)
        }
    }

    /// Get the users matching the given relationship, the regular friends list by default.
    /// `flags` is a bitmask of `FriendFlags` values, which may be OR-ed together.
    /// See [Steam API](https://partner.steamgames.com/doc/api/ISteamFriends#GetFriendByIndex)
    #[napi]
    pub fn get_friends(flags: Option<u32>) -> Vec<Friend> {
        let client = crate::client::get_client();
        let flags = flags
            .map(friend_flags_from_bits)
            .unwrap_or(steamworks::FriendFlags::IMMEDIATE);
        client
            .friends()
            .get_friends(flags)
            .into_iter()
            .map(|friend| Friend::from_steamid(friend.id()))
            .collect()
    }

    /// Get the users on the local user's recently-played-with list.
    /// See [Steam API](https://partner.steamgames.com/doc/api/ISteamFriends#GetCoplayFriend)
    #[napi]
    pub fn get_coplay_friends() -> Vec<Friend> {
        let client = crate::client::get_client();
        client
            .friends()
            .get_coplay_friends()
            .into_iter()
            .map(|friend| Friend::from_steamid(friend.id()))
            .collect()
    }

    /// Get an arbitrary user by steam id, they don't have to be a friend.
    #[napi]
    pub fn get_friend(steam_id64: BigInt) -> Friend {
        Friend::from_steamid(SteamId::from_raw(steam_id64.get_u64().1))
    }

    /// Request the persona name and optionally the avatar of a user from Steam.
    /// @param nameOnly - Only request the name, skipping the avatar.
    /// @returns true if the information is being fetched, false if it was already available.
    /// See [Steam API](https://partner.steamgames.com/doc/api/ISteamFriends#RequestUserInformation)
    #[napi]
    pub fn request_user_information(steam_id64: BigInt, name_only: bool) -> bool {
        let client = crate::client::get_client();
        client
            .friends()
            .request_user_information(SteamId::from_raw(steam_id64.get_u64().1), name_only)
    }
}
