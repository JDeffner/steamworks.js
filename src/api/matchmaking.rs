use napi_derive::napi;

#[napi]
pub mod matchmaking {
    use crate::api::localplayer::PlayerSteamId;
    use napi::bindgen_prelude::{BigInt, Error};
    use std::collections::HashMap;
    use steamworks::LobbyId;
    use tokio::sync::oneshot;

    #[napi]
    pub enum LobbyType {
        Private,
        FriendsOnly,
        Public,
        Invisible,
    }

    /// Comparison used by a string lobby list filter.
    /// @see https://partner.steamgames.com/doc/api/ISteamMatchmaking#ELobbyComparison
    #[napi]
    pub enum LobbyStringComparison {
        EqualToOrLessThan,
        LessThan,
        Equal,
        GreaterThan,
        EqualToOrGreaterThan,
        NotEqual,
    }

    /// Comparison used by a numerical lobby list filter.
    /// @see https://partner.steamgames.com/doc/api/ISteamMatchmaking#ELobbyComparison
    #[napi]
    pub enum LobbyNumberComparison {
        Equal,
        NotEqual,
        GreaterThan,
        GreaterThanEqualTo,
        LessThan,
        LessThanEqualTo,
    }

    /// How far geographically the returned lobbies may be.
    /// @see https://partner.steamgames.com/doc/api/ISteamMatchmaking#ELobbyDistanceFilter
    #[napi]
    pub enum LobbyDistanceFilter {
        Close,
        Default,
        Far,
        Worldwide,
    }

    /// Matches lobbies whose string metadata compares against `value` as requested.
    /// @see https://partner.steamgames.com/doc/api/ISteamMatchmaking#AddRequestLobbyListStringFilter
    #[napi(object)]
    pub struct LobbyStringFilter {
        pub key: String,
        pub value: String,
        pub comparison: LobbyStringComparison,
    }

    /// Matches lobbies whose numerical metadata compares against `value` as requested.
    /// @see https://partner.steamgames.com/doc/api/ISteamMatchmaking#AddRequestLobbyListNumericalFilter
    #[napi(object)]
    pub struct LobbyNumberFilter {
        pub key: String,
        pub value: i32,
        pub comparison: LobbyNumberComparison,
    }

    /// Sorts the results by how close their metadata is to `value`. This does not filter anything out.
    /// @see https://partner.steamgames.com/doc/api/ISteamMatchmaking#AddRequestLobbyListNearValueFilter
    #[napi(object)]
    pub struct LobbyNearFilter {
        pub key: String,
        pub value: i32,
    }

    /// Filters applied to a lobby list request. Every field is optional; an empty
    /// filter returns the same lobbies as an unfiltered request.
    /// @see https://partner.steamgames.com/doc/api/ISteamMatchmaking#RequestLobbyList
    #[napi(object)]
    pub struct LobbyListFilter {
        /// String metadata comparisons a lobby has to satisfy
        pub string_filters: Option<Vec<LobbyStringFilter>>,
        /// Numerical metadata comparisons a lobby has to satisfy
        pub number_filters: Option<Vec<LobbyNumberFilter>>,
        /// Metadata values the results are sorted closest to
        pub near_value_filters: Option<Vec<LobbyNearFilter>>,
        /// Only return lobbies with at least this many open slots (0-255)
        pub slots_available: Option<u8>,
        /// How far geographically the returned lobbies may be
        pub distance: Option<LobbyDistanceFilter>,
        /// Maximum amount of lobbies to return
        pub count: Option<u32>,
    }

    impl From<&LobbyStringComparison> for steamworks::StringFilterKind {
        fn from(value: &LobbyStringComparison) -> Self {
            match value {
                LobbyStringComparison::EqualToOrLessThan => {
                    steamworks::StringFilterKind::EqualToOrLessThan
                }
                LobbyStringComparison::LessThan => steamworks::StringFilterKind::LessThan,
                LobbyStringComparison::Equal => steamworks::StringFilterKind::Equal,
                LobbyStringComparison::GreaterThan => steamworks::StringFilterKind::GreaterThan,
                LobbyStringComparison::EqualToOrGreaterThan => {
                    steamworks::StringFilterKind::EqualToOrGreaterThan
                }
                LobbyStringComparison::NotEqual => steamworks::StringFilterKind::NotEqual,
            }
        }
    }

    impl From<&LobbyNumberComparison> for steamworks::ComparisonFilter {
        fn from(value: &LobbyNumberComparison) -> Self {
            match value {
                LobbyNumberComparison::Equal => steamworks::ComparisonFilter::Equal,
                LobbyNumberComparison::NotEqual => steamworks::ComparisonFilter::NotEqual,
                LobbyNumberComparison::GreaterThan => steamworks::ComparisonFilter::GreaterThan,
                LobbyNumberComparison::GreaterThanEqualTo => {
                    steamworks::ComparisonFilter::GreaterThanEqualTo
                }
                LobbyNumberComparison::LessThan => steamworks::ComparisonFilter::LessThan,
                LobbyNumberComparison::LessThanEqualTo => {
                    steamworks::ComparisonFilter::LessThanEqualTo
                }
            }
        }
    }

    impl From<&LobbyDistanceFilter> for steamworks::DistanceFilter {
        fn from(value: &LobbyDistanceFilter) -> Self {
            match value {
                LobbyDistanceFilter::Close => steamworks::DistanceFilter::Close,
                LobbyDistanceFilter::Default => steamworks::DistanceFilter::Default,
                LobbyDistanceFilter::Far => steamworks::DistanceFilter::Far,
                LobbyDistanceFilter::Worldwide => steamworks::DistanceFilter::Worldwide,
            }
        }
    }

    /// Steam takes lobby keys and values as C strings of at most 255 bytes, and the
    /// underlying crate panics on anything else, so validate before handing them over.
    fn lobby_filter_key(key: &str) -> Result<steamworks::LobbyKey<'_>, Error> {
        if key.contains('\0') {
            return Err(Error::from_reason(format!(
                "Lobby filter key \"{}\" contains a null byte",
                key.escape_debug()
            )));
        }

        steamworks::LobbyKey::try_new(key).map_err(|e| Error::from_reason(e.to_string()))
    }

    fn lobby_filter_value(value: &str) -> Result<&str, Error> {
        if value.contains('\0') {
            return Err(Error::from_reason(format!(
                "Lobby filter value \"{}\" contains a null byte",
                value.escape_debug()
            )));
        }

        Ok(value)
    }

    #[napi]
    pub struct Lobby {
        pub id: BigInt,
        lobby_id: LobbyId,
    }

    #[napi]
    impl Lobby {
        #[napi]
        pub async fn join(&self) -> Result<Lobby, Error> {
            join_lobby(self.id.clone()).await
        }

        #[napi]
        pub fn leave(&self) {
            let client = crate::client::get_client();
            client.matchmaking().leave_lobby(self.lobby_id);
        }

        #[napi]
        pub fn open_invite_dialog(&self) {
            let client = crate::client::get_client();
            client.friends().activate_invite_dialog(self.lobby_id);
        }

        #[napi]
        pub fn get_member_count(&self) -> usize {
            let client = crate::client::get_client();
            client.matchmaking().lobby_member_count(self.lobby_id)
        }

        #[napi]
        pub fn get_member_limit(&self) -> Option<usize> {
            let client = crate::client::get_client();
            client.matchmaking().lobby_member_limit(self.lobby_id)
        }

        #[napi]
        pub fn get_members(&self) -> Vec<PlayerSteamId> {
            let client = crate::client::get_client();
            client
                .matchmaking()
                .lobby_members(self.lobby_id)
                .into_iter()
                .map(PlayerSteamId::from_steamid)
                .collect()
        }

        #[napi]
        pub fn get_owner(&self) -> PlayerSteamId {
            let client = crate::client::get_client();
            PlayerSteamId::from_steamid(client.matchmaking().lobby_owner(self.lobby_id))
        }

        #[napi]
        pub fn set_joinable(&self, joinable: bool) -> bool {
            let client = crate::client::get_client();
            client
                .matchmaking()
                .set_lobby_joinable(self.lobby_id, joinable)
        }

        #[napi]
        pub fn get_data(&self, key: String) -> Option<String> {
            let client = crate::client::get_client();
            client
                .matchmaking()
                .lobby_data(self.lobby_id, &key)
                .map(|s| s.to_string())
        }

        #[napi]
        pub fn set_data(&self, key: String, value: String) -> bool {
            let client = crate::client::get_client();
            client
                .matchmaking()
                .set_lobby_data(self.lobby_id, &key, &value)
        }

        #[napi]
        pub fn delete_data(&self, key: String) -> bool {
            let client = crate::client::get_client();
            client.matchmaking().delete_lobby_data(self.lobby_id, &key)
        }

        /// Get an object containing all the lobby data
        #[napi]
        pub fn get_full_data(&self) -> HashMap<String, String> {
            let client = crate::client::get_client();

            let mut data = HashMap::new();

            let count = client.matchmaking().lobby_data_count(self.lobby_id);
            for i in 0..count {
                let maybe_lobby_data = client.matchmaking().lobby_data_by_index(self.lobby_id, i);

                if let Some((key, value)) = maybe_lobby_data {
                    data.insert(key, value);
                }
            }

            data
        }

        /// Merge current lobby data with provided data in a single batch
        /// @returns true if all data was set successfully
        #[napi]
        pub fn merge_full_data(&self, data: HashMap<String, String>) -> bool {
            let matchmaking = crate::client::get_client().matchmaking();
            data.iter()
                .map(|(key, value)| matchmaking.set_lobby_data(self.lobby_id, key, value))
                .all(|x| x)
        }
    }

    #[napi]
    pub async fn create_lobby(lobby_type: LobbyType, max_members: u32) -> Result<Lobby, Error> {
        let client = crate::client::get_client();

        let (tx, rx) = oneshot::channel();

        client.matchmaking().create_lobby(
            match lobby_type {
                LobbyType::Private => steamworks::LobbyType::Private,
                LobbyType::FriendsOnly => steamworks::LobbyType::FriendsOnly,
                LobbyType::Public => steamworks::LobbyType::Public,
                LobbyType::Invisible => steamworks::LobbyType::Invisible,
            },
            max_members,
            |result| {
                tx.send(result).unwrap();
            },
        );

        rx.await
            .unwrap()
            .map(|lobby_id| Lobby {
                id: BigInt::from(lobby_id.raw()),
                lobby_id,
            })
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    #[napi]
    pub async fn join_lobby(lobby_id: BigInt) -> Result<Lobby, Error> {
        let client = crate::client::get_client();

        let (tx, rx) = oneshot::channel();

        client.matchmaking().join_lobby(
            steamworks::LobbyId::from_raw(lobby_id.get_u64().1),
            |result| {
                tx.send(result).unwrap();
            },
        );

        rx.await
            .unwrap()
            .map(|lobby_id| Lobby {
                id: BigInt::from(lobby_id.raw()),
                lobby_id,
            })
            .map_err(|_| Error::from_reason("Failed to join lobby".to_string()))
    }

    /// Get the list of lobbies for this app, optionally narrowed down by `filter`.
    /// Calling this without a filter returns the unfiltered lobby list.
    /// @see https://partner.steamgames.com/doc/api/ISteamMatchmaking#RequestLobbyList
    #[napi]
    pub async fn get_lobbies(filter: Option<LobbyListFilter>) -> Result<Vec<Lobby>, Error> {
        let client = crate::client::get_client();

        let (tx, rx) = oneshot::channel();

        {
            // The filters are applied to the same matchmaking accessor that issues the
            // request, and only live until `request_lobby_list` consumes them, matching
            // the steamworks-rs example. Keeping the accessor out of the await below
            // also keeps this future `Send`.
            let matchmaking = client.matchmaking();

            match &filter {
                Some(filter) => {
                    let string = filter
                        .string_filters
                        .iter()
                        .flatten()
                        .map(|f| {
                            Ok(steamworks::StringFilter(
                                lobby_filter_key(&f.key)?,
                                lobby_filter_value(&f.value)?,
                                (&f.comparison).into(),
                            ))
                        })
                        .collect::<Result<Vec<_>, Error>>()?;

                    let number = filter
                        .number_filters
                        .iter()
                        .flatten()
                        .map(|f| {
                            Ok(steamworks::NumberFilter(
                                lobby_filter_key(&f.key)?,
                                f.value,
                                (&f.comparison).into(),
                            ))
                        })
                        .collect::<Result<Vec<_>, Error>>()?;

                    let near_value = filter
                        .near_value_filters
                        .iter()
                        .flatten()
                        .map(|f| Ok(steamworks::NearFilter(lobby_filter_key(&f.key)?, f.value)))
                        .collect::<Result<Vec<_>, Error>>()?;

                    matchmaking
                        .set_lobby_list_filter(steamworks::LobbyListFilter {
                            string: Some(string),
                            number: Some(number),
                            near_value: Some(near_value),
                            open_slots: filter.slots_available,
                            distance: filter.distance.as_ref().map(Into::into),
                            count: filter.count.map(u64::from),
                        })
                        .request_lobby_list(|lobbies| {
                            tx.send(lobbies).unwrap();
                        });
                }
                None => matchmaking.request_lobby_list(|lobbies| {
                    tx.send(lobbies).unwrap();
                }),
            }
        }

        rx.await
            .unwrap()
            .map(|lobbies| {
                lobbies
                    .iter()
                    .map(|lobby_id| Lobby {
                        id: BigInt::from(lobby_id.raw()),
                        lobby_id: *lobby_id,
                    })
                    .collect()
            })
            .map_err(|e| Error::from_reason(e.to_string()))
    }
}
