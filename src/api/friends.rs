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

    impl From<FriendFlags> for steamworks::FriendFlags {
        fn from(flags: FriendFlags) -> Self {
            match flags {
                FriendFlags::None => steamworks::FriendFlags::NONE,
                FriendFlags::Blocked => steamworks::FriendFlags::BLOCKED,
                FriendFlags::FriendshipRequested => steamworks::FriendFlags::FRIENDSHIP_REQUESTED,
                FriendFlags::Immediate => steamworks::FriendFlags::IMMEDIATE,
                FriendFlags::ClanMember => steamworks::FriendFlags::CLAN_MEMBER,
                FriendFlags::OnGameServer => steamworks::FriendFlags::ON_GAME_SERVER,
                FriendFlags::RequestingFriendship => steamworks::FriendFlags::REQUESTING_FRIENDSHIP,
                FriendFlags::RequestingInfo => steamworks::FriendFlags::REQUESTING_INFO,
                FriendFlags::Ignored => steamworks::FriendFlags::IGNORED,
                FriendFlags::IgnoredFriend => steamworks::FriendFlags::IGNORED_FRIEND,
                FriendFlags::ChatMember => steamworks::FriendFlags::CHAT_MEMBER,
                FriendFlags::All => steamworks::FriendFlags::ALL,
            }
        }
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
    }

    impl From<steamworks::FriendState> for FriendState {
        fn from(state: steamworks::FriendState) -> Self {
            match state {
                steamworks::FriendState::Offline => FriendState::Offline,
                steamworks::FriendState::Online => FriendState::Online,
                steamworks::FriendState::Busy => FriendState::Busy,
                steamworks::FriendState::Away => FriendState::Away,
                steamworks::FriendState::Snooze => FriendState::Snooze,
                steamworks::FriendState::LookingToTrade => FriendState::LookingToTrade,
                steamworks::FriendState::LookingToPlay => FriendState::LookingToPlay,
            }
        }
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
            let client = crate::client::get_client();
            client.friends().get_friend(self.steam_id).state().into()
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
        #[napi]
        pub fn has_friend(&self, flags: FriendFlags) -> bool {
            let client = crate::client::get_client();
            client
                .friends()
                .get_friend(self.steam_id)
                .has_friend(flags.into())
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
    /// See [Steam API](https://partner.steamgames.com/doc/api/ISteamFriends#GetFriendByIndex)
    #[napi]
    pub fn get_friends(flags: Option<FriendFlags>) -> Vec<Friend> {
        let client = crate::client::get_client();
        client
            .friends()
            .get_friends(flags.unwrap_or(FriendFlags::Immediate).into())
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
