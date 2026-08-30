use napi_derive::napi;

#[napi]
pub mod leaderboard {
    use crate::api::localplayer::PlayerSteamId;
    use napi::bindgen_prelude::{BigInt, Error};
    use tokio::sync::oneshot;

    /// The sort order of a leaderboard.
    /// {@link https://partner.steamgames.com/doc/api/ISteamUserStats#ELeaderboardSortMethod}
    #[napi]
    pub enum LeaderboardSortMethod {
        /// The top-score is the lowest number.
        Ascending,
        /// The top-score is the highest number.
        Descending,
    }

    impl From<LeaderboardSortMethod> for steamworks::LeaderboardSortMethod {
        fn from(val: LeaderboardSortMethod) -> Self {
            match val {
                LeaderboardSortMethod::Ascending => steamworks::LeaderboardSortMethod::Ascending,
                LeaderboardSortMethod::Descending => steamworks::LeaderboardSortMethod::Descending,
            }
        }
    }

    impl From<steamworks::LeaderboardSortMethod> for LeaderboardSortMethod {
        fn from(val: steamworks::LeaderboardSortMethod) -> Self {
            match val {
                steamworks::LeaderboardSortMethod::Ascending => LeaderboardSortMethod::Ascending,
                steamworks::LeaderboardSortMethod::Descending => LeaderboardSortMethod::Descending,
            }
        }
    }

    /// How a leaderboard score is displayed in the Steam overlay and community.
    /// {@link https://partner.steamgames.com/doc/api/ISteamUserStats#ELeaderboardDisplayType}
    #[napi]
    pub enum LeaderboardDisplayType {
        /// The score is just a simple numerical value.
        Numeric,
        /// The score represents a time, in seconds.
        TimeSeconds,
        /// The score represents a time, in milliseconds.
        TimeMilliSeconds,
    }

    impl From<LeaderboardDisplayType> for steamworks::LeaderboardDisplayType {
        fn from(val: LeaderboardDisplayType) -> Self {
            match val {
                LeaderboardDisplayType::Numeric => steamworks::LeaderboardDisplayType::Numeric,
                LeaderboardDisplayType::TimeSeconds => {
                    steamworks::LeaderboardDisplayType::TimeSeconds
                }
                LeaderboardDisplayType::TimeMilliSeconds => {
                    steamworks::LeaderboardDisplayType::TimeMilliSeconds
                }
            }
        }
    }

    impl From<steamworks::LeaderboardDisplayType> for LeaderboardDisplayType {
        fn from(val: steamworks::LeaderboardDisplayType) -> Self {
            match val {
                steamworks::LeaderboardDisplayType::Numeric => LeaderboardDisplayType::Numeric,
                steamworks::LeaderboardDisplayType::TimeSeconds => {
                    LeaderboardDisplayType::TimeSeconds
                }
                steamworks::LeaderboardDisplayType::TimeMilliSeconds => {
                    LeaderboardDisplayType::TimeMilliSeconds
                }
            }
        }
    }

    /// How an uploaded score is treated when the user already has an entry.
    /// {@link https://partner.steamgames.com/doc/api/ISteamUserStats#ELeaderboardUploadScoreMethod}
    #[napi]
    pub enum LeaderboardUploadScoreMethod {
        /// Only replaces the existing entry if the new score is better.
        KeepBest,
        /// Always replaces the existing entry, even with a worse score.
        ForceUpdate,
    }

    impl From<LeaderboardUploadScoreMethod> for steamworks::UploadScoreMethod {
        fn from(val: LeaderboardUploadScoreMethod) -> Self {
            match val {
                LeaderboardUploadScoreMethod::KeepBest => steamworks::UploadScoreMethod::KeepBest,
                LeaderboardUploadScoreMethod::ForceUpdate => {
                    steamworks::UploadScoreMethod::ForceUpdate
                }
            }
        }
    }

    /// Which set of leaderboard entries to download.
    /// {@link https://partner.steamgames.com/doc/api/ISteamUserStats#ELeaderboardDataRequest}
    #[napi]
    pub enum LeaderboardDataRequest {
        /// Query everyone on the leaderboard, `start` and `end` are absolute ranks (1 based).
        Global,
        /// Query around the current user, `start` and `end` are relative to the user's rank.
        GlobalAroundUser,
        /// Query the current user's friends, `start` and `end` are ignored.
        Friends,
    }

    impl From<LeaderboardDataRequest> for steamworks::LeaderboardDataRequest {
        fn from(val: LeaderboardDataRequest) -> Self {
            match val {
                LeaderboardDataRequest::Global => steamworks::LeaderboardDataRequest::Global,
                LeaderboardDataRequest::GlobalAroundUser => {
                    steamworks::LeaderboardDataRequest::GlobalAroundUser
                }
                LeaderboardDataRequest::Friends => steamworks::LeaderboardDataRequest::Friends,
            }
        }
    }

    /// The outcome of a successful score upload.
    /// {@link https://partner.steamgames.com/doc/api/ISteamUserStats#LeaderboardScoreUploaded_t}
    #[napi(object)]
    pub struct LeaderboardScoreUploaded {
        /// The score that was submitted.
        pub score: i32,
        /// Whether the score on the leaderboard actually changed.
        /// False when `KeepBest` was used and the existing score was better.
        pub score_changed: bool,
        /// The new global rank of the user, 0 when the score did not change.
        pub global_rank_new: i32,
        /// The global rank the user had before this upload, 0 when they had no entry.
        pub global_rank_previous: i32,
    }

    /// A single downloaded leaderboard entry.
    /// {@link https://partner.steamgames.com/doc/api/ISteamUserStats#LeaderboardEntry_t}
    #[napi(object)]
    pub struct LeaderboardEntry {
        /// The user that owns this entry.
        pub steam_id: PlayerSteamId,
        /// The global rank of this entry, 1 based.
        pub global_rank: i32,
        /// The score of this entry.
        pub score: i32,
        /// The game specific details uploaded with the score.
        /// Empty unless `maxDetailsLen` was greater than 0 when downloading.
        pub details: Vec<i32>,
    }

    /// A handle to a Steam leaderboard, obtained through {@link findLeaderboard} or
    /// {@link findOrCreateLeaderboard}.
    /// {@link https://partner.steamgames.com/doc/api/ISteamUserStats}
    #[napi]
    pub struct Leaderboard {
        leaderboard: steamworks::Leaderboard,
    }

    impl Leaderboard {
        pub(crate) fn from_handle(leaderboard: steamworks::Leaderboard) -> Self {
            Self { leaderboard }
        }
    }

    // The crate unwraps CString::new on leaderboard names, so an interior null byte
    // would otherwise panic inside the spawned future instead of rejecting cleanly.
    fn leaderboard_name(name: &str) -> Result<&str, Error> {
        if name.contains('\0') {
            Err(Error::from_reason(
                "Leaderboard name contains a null byte".to_string(),
            ))
        } else {
            Ok(name)
        }
    }

    #[napi]
    impl Leaderboard {
        /// The raw `SteamLeaderboard_t` handle.
        #[napi(getter)]
        pub fn handle(&self) -> BigInt {
            BigInt::from(self.leaderboard.raw())
        }
        /// Get the name of this leaderboard.
        /// {@link https://partner.steamgames.com/doc/api/ISteamUserStats#GetLeaderboardName}
        #[napi]
        pub fn get_name(&self) -> String {
            let client = crate::client::get_client();
            client.user_stats().get_leaderboard_name(&self.leaderboard)
        }

        /// Get the total number of entries in this leaderboard.
        /// {@link https://partner.steamgames.com/doc/api/ISteamUserStats#GetLeaderboardEntryCount}
        #[napi]
        pub fn get_entry_count(&self) -> i32 {
            let client = crate::client::get_client();
            client
                .user_stats()
                .get_leaderboard_entry_count(&self.leaderboard)
        }

        /// Get the sort method of this leaderboard, or null if the handle is invalid.
        /// {@link https://partner.steamgames.com/doc/api/ISteamUserStats#GetLeaderboardSortMethod}
        #[napi]
        pub fn get_sort_method(&self) -> Option<LeaderboardSortMethod> {
            let client = crate::client::get_client();
            client
                .user_stats()
                .get_leaderboard_sort_method(&self.leaderboard)
                .map(LeaderboardSortMethod::from)
        }

        /// Get the display type of this leaderboard, or null if the handle is invalid.
        /// {@link https://partner.steamgames.com/doc/api/ISteamUserStats#GetLeaderboardDisplayType}
        #[napi]
        pub fn get_display_type(&self) -> Option<LeaderboardDisplayType> {
            let client = crate::client::get_client();
            client
                .user_stats()
                .get_leaderboard_display_type(&self.leaderboard)
                .map(LeaderboardDisplayType::from)
        }

        /// Upload a score to this leaderboard for the current user.
        ///
        /// `details` is optional game specific data (at most 64 entries) stored alongside the
        /// score, for example a replay seed or a per-level breakdown.
        /// {@link https://partner.steamgames.com/doc/api/ISteamUserStats#UploadLeaderboardScore}
        #[napi]
        pub async fn upload_score(
            &self,
            score: i32,
            method: LeaderboardUploadScoreMethod,
            details: Option<Vec<i32>>,
        ) -> Result<LeaderboardScoreUploaded, Error> {
            let details = details.unwrap_or_default();
            let max_details = steamworks::sys::k_cLeaderboardDetailsMax as usize;
            if details.len() > max_details {
                // The SDK rejects oversized uploads with an invalid call handle, which
                // would leave the promise pending forever instead of erroring.
                return Err(Error::from_reason(format!(
                    "details supports at most {} entries, got {}",
                    max_details,
                    details.len()
                )));
            }

            let client = crate::client::get_client();

            let (tx, rx) = oneshot::channel();

            client.user_stats().upload_leaderboard_score(
                &self.leaderboard,
                method.into(),
                score,
                &details,
                |result| {
                    tx.send(result).unwrap();
                },
            );

            match rx.await.unwrap() {
                Ok(Some(result)) => Ok(LeaderboardScoreUploaded {
                    score: result.score,
                    score_changed: result.was_changed,
                    global_rank_new: result.global_rank_new,
                    global_rank_previous: result.global_rank_previous,
                }),
                Ok(None) => Err(Error::from_reason(
                    "Failed to upload leaderboard score".to_string(),
                )),
                Err(e) => Err(Error::from_reason(e.to_string())),
            }
        }

        /// Download a range of entries from this leaderboard.
        ///
        /// The meaning of `start` and `end` depends on `request`: absolute 1 based ranks for
        /// `Global`, offsets relative to the current user for `GlobalAroundUser` (where `start`
        /// is usually negative, e.g. -4 to 5 for the ten entries around the user), and ignored
        /// for `Friends`. `maxDetailsLen` is how many details entries to read per row (0 to 64),
        /// pass 0 when the leaderboard has no details.
        /// {@link https://partner.steamgames.com/doc/api/ISteamUserStats#DownloadLeaderboardEntries}
        #[napi]
        pub async fn download_entries(
            &self,
            request: LeaderboardDataRequest,
            start: i32,
            end: i32,
            max_details_len: u32,
        ) -> Result<Vec<LeaderboardEntry>, Error> {
            let client = crate::client::get_client();

            let (tx, rx) = oneshot::channel();

            client.user_stats().download_leaderboard_entries(
                &self.leaderboard,
                request.into(),
                // The crate takes usize but hands the value straight to the SDK as a c_int, so
                // sign extending here round-trips the negative offsets GlobalAroundUser needs.
                start as usize,
                end as usize,
                max_details_len as usize,
                |result| {
                    tx.send(result).unwrap();
                },
            );

            rx.await
                .unwrap()
                .map(|entries| {
                    entries
                        .into_iter()
                        .map(|entry| LeaderboardEntry {
                            steam_id: PlayerSteamId::from_steamid(entry.user),
                            global_rank: entry.global_rank,
                            score: entry.score,
                            details: entry.details,
                        })
                        .collect()
                })
                .map_err(|e| Error::from_reason(e.to_string()))
        }
    }

    /// Find a leaderboard by its name, as configured on the Steamworks partner site.
    ///
    /// Resolves to null when no leaderboard with that name exists.
    /// {@link https://partner.steamgames.com/doc/api/ISteamUserStats#FindLeaderboard}
    #[napi]
    pub async fn find_leaderboard(name: String) -> Result<Option<Leaderboard>, Error> {
        let client = crate::client::get_client();

        let (tx, rx) = oneshot::channel();

        client
            .user_stats()
            .find_leaderboard(leaderboard_name(&name)?, |result| {
                tx.send(result).unwrap();
            });

        rx.await
            .unwrap()
            .map(|leaderboard| leaderboard.map(Leaderboard::from_handle))
            .map_err(|e| Error::from_reason(e.to_string()))
    }

    /// Find a leaderboard by name, creating it if it does not exist yet.
    ///
    /// The sort method and display type are only used when the leaderboard is created; an
    /// existing leaderboard keeps the settings it was created with.
    /// {@link https://partner.steamgames.com/doc/api/ISteamUserStats#FindOrCreateLeaderboard}
    #[napi]
    pub async fn find_or_create_leaderboard(
        name: String,
        sort_method: LeaderboardSortMethod,
        display_type: LeaderboardDisplayType,
    ) -> Result<Leaderboard, Error> {
        let client = crate::client::get_client();

        let (tx, rx) = oneshot::channel();

        client.user_stats().find_or_create_leaderboard(
            leaderboard_name(&name)?,
            sort_method.into(),
            display_type.into(),
            |result| {
                tx.send(result).unwrap();
            },
        );

        match rx.await.unwrap() {
            Ok(Some(leaderboard)) => Ok(Leaderboard::from_handle(leaderboard)),
            Ok(None) => Err(Error::from_reason(
                "Failed to find or create leaderboard".to_string(),
            )),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }
}
