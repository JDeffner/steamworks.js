use napi_derive::napi;

#[napi]
pub mod workshop {
    use crate::api::FatalTsfn;
    use napi::bindgen_prelude::{BigInt, Error};
    use napi::threadsafe_function::ThreadsafeFunctionCallMode;
    use std::path::Path;
    use steamworks::{ClientManager, FileType, PublishedFileId, UpdateHandle};
    use tokio::sync::oneshot;

    fn pfid(b: BigInt) -> PublishedFileId {
        PublishedFileId(b.get_u64().1)
    }

    #[napi(object)]
    pub struct UgcResult {
        pub item_id: BigInt,
        pub needs_to_accept_agreement: bool,
    }

    #[derive(Debug)]
    #[napi]
    pub enum UgcItemVisibility {
        Public,
        FriendsOnly,
        Private,
        Unlisted,
    }

    crate::api::impl_enum_from!(steamworks::PublishedFileVisibility => UgcItemVisibility {
        Public, FriendsOnly, Private, Unlisted,
    });
    crate::api::impl_enum_from!(UgcItemVisibility => steamworks::PublishedFileVisibility {
        Public, FriendsOnly, Private, Unlisted,
    });

    #[napi(object)]
    pub struct UgcUpdate {
        pub title: Option<String>,
        pub description: Option<String>,
        pub change_note: Option<String>,
        pub preview_path: Option<String>,
        pub content_path: Option<String>,
        pub tags: Option<Vec<String>>,
        pub visibility: Option<UgcItemVisibility>,
    }

    impl UgcUpdate {
        pub fn submit(
            self,
            mut update: UpdateHandle<ClientManager>,
            callback: impl FnOnce(Result<(PublishedFileId, bool), steamworks::SteamError>)
                + Send
                + 'static,
        ) -> steamworks::UpdateWatchHandle<ClientManager> {
            if let Some(title) = self.title {
                update = update.title(title.as_str());
            }

            if let Some(description) = self.description {
                update = update.description(description.as_str());
            }

            if let Some(preview_path) = self.preview_path {
                update = update.preview_path(Path::new(&preview_path));
            }

            if let Some(tags) = self.tags {
                update = update.tags(tags, false);
            }

            if let Some(content_path) = self.content_path {
                update = update.content_path(Path::new(&content_path));
            }

            if let Some(visibility) = self.visibility {
                update = update.visibility(visibility.into());
            }

            let change_note = self.change_note.as_deref();
            update.submit(change_note, callback)
        }
    }

    #[napi(object)]
    pub struct InstallInfo {
        pub folder: String,
        pub size_on_disk: BigInt,
        pub timestamp: u32,
    }

    #[napi(object)]
    pub struct DownloadInfo {
        pub current: BigInt,
        pub total: BigInt,
    }

    #[derive(Debug)]
    #[napi]
    pub enum UpdateStatus {
        Invalid,
        PreparingConfig,
        PreparingContent,
        UploadingContent,
        UploadingPreviewFile,
        CommittingChanges,
    }

    crate::api::impl_enum_from!(steamworks::UpdateStatus => UpdateStatus {
        Invalid, PreparingConfig, PreparingContent,
        UploadingContent, UploadingPreviewFile, CommittingChanges,
    });

    #[napi(object)]
    pub struct UpdateProgress {
        pub status: UpdateStatus,
        pub progress: BigInt,
        pub total: BigInt,
    }

    #[napi]
    pub async fn create_item(app_id: Option<u32>) -> Result<UgcResult, Error> {
        let client = crate::client::get_client();
        let app_id = app_id
            .map(steamworks::AppId)
            .unwrap_or_else(|| client.utils().app_id());

        let (tx, rx) = oneshot::channel();

        client
            .ugc()
            .create_item(app_id, FileType::Community, |result| {
                tx.send(result).unwrap();
            });

        let result = rx.await.unwrap();
        match result {
            Ok((item_id, needs_to_accept_agreement)) => Ok(UgcResult {
                item_id: BigInt::from(item_id.0),
                needs_to_accept_agreement,
            }),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }

    #[napi]
    pub async fn update_item(
        item_id: BigInt,
        update_details: UgcUpdate,
        app_id: Option<u32>,
    ) -> Result<UgcResult, Error> {
        let client = crate::client::get_client();

        let app_id = app_id
            .map(steamworks::AppId)
            .unwrap_or_else(|| client.utils().app_id());

        let (tx, rx) = oneshot::channel();

        {
            let update_handle = client
                .ugc()
                .start_item_update(app_id, pfid(item_id));

            update_details.submit(update_handle, |result| {
                tx.send(result).unwrap();
            });
        };

        let result = rx.await.unwrap();
        match result {
            Ok((item_id, needs_to_accept_agreement)) => Ok(UgcResult {
                item_id: BigInt::from(item_id.0),
                needs_to_accept_agreement,
            }),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }

    #[napi]
    pub fn update_item_with_callback(
        item_id: BigInt,
        update_details: UgcUpdate,
        app_id: Option<u32>,

        #[napi(ts_arg_type = "(data: UgcResult) => void")] success_callback: FatalTsfn<UgcResult>,

        #[napi(ts_arg_type = "(err: any) => void")] error_callback: FatalTsfn<Error>,

        #[napi(ts_arg_type = "(data: UpdateProgress) => void")] progress_callback: Option<
            FatalTsfn<UpdateProgress>,
        >,

        progress_callback_interval_ms: Option<u32>,
    ) {

        let client = crate::client::get_client();

        let app_id = app_id
            .map(steamworks::AppId)
            .unwrap_or_else(|| client.utils().app_id());

        {
            let update_handle = client
                .ugc()
                .start_item_update(app_id, pfid(item_id));

            let update_watch_handle = update_details.submit(update_handle, move |result| {
                match result {
                    Ok((item_id, needs_to_accept_agreement)) => success_callback.call(
                        UgcResult {
                            item_id: BigInt::from(item_id.0),
                            needs_to_accept_agreement,
                        },
                        ThreadsafeFunctionCallMode::Blocking,
                    ),
                    Err(e) => error_callback.call(
                        Error::from_reason(e.to_string()),
                        ThreadsafeFunctionCallMode::Blocking,
                    ),
                };
            });

            if let Some(progress_callback) = progress_callback {
                std::thread::spawn(move || loop {
                    let (status, progress, total) = update_watch_handle.progress();
                    let value = UpdateProgress {
                        status: status.into(),
                        progress: BigInt::from(progress),
                        total: BigInt::from(total),
                    };
                    progress_callback.call(value, ThreadsafeFunctionCallMode::Blocking);
                    match status {
                        steamworks::UpdateStatus::Invalid => break,
                        steamworks::UpdateStatus::CommittingChanges => break,
                        _ => (),
                    }
                    std::thread::sleep(std::time::Duration::from_millis(
                        progress_callback_interval_ms.unwrap_or(1000) as u64,
                    ));
                });
            }
        };
    }

    /// Subscribe to a workshop item. It will be downloaded and installed as soon as possible.
    ///
    /// {@link https://partner.steamgames.com/doc/api/ISteamUGC#SubscribeItem}
    #[napi]
    pub async fn subscribe(item_id: BigInt) -> Result<(), Error> {
        let client = crate::client::get_client();
        let (tx, rx) = oneshot::channel();

        client
            .ugc()
            .subscribe_item(pfid(item_id), |result| {
                tx.send(result).unwrap();
            });

        let result = rx.await.unwrap();
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }

    /// Unsubscribe from a workshop item. This will result in the item being removed after the game quits.
    ///
    /// {@link https://partner.steamgames.com/doc/api/ISteamUGC#UnsubscribeItem}
    #[napi]
    pub async fn unsubscribe(item_id: BigInt) -> Result<(), Error> {
        let client = crate::client::get_client();
        let (tx, rx) = oneshot::channel();

        client
            .ugc()
            .unsubscribe_item(pfid(item_id), |result| {
                tx.send(result).unwrap();
            });

        let result = rx.await.unwrap();
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }

    /// Gets the current state of a workshop item on this client. States can be combined.
    ///
    /// @returns a number with the current item state, e.g. 9
    /// 9 = 1 (The current user is subscribed to this item) + 8 (The item needs an update)
    ///
    /// {@link https://partner.steamgames.com/doc/api/ISteamUGC#GetItemState}
    /// {@link https://partner.steamgames.com/doc/api/ISteamUGC#EItemState}
    #[napi]
    pub fn state(item_id: BigInt) -> u32 {
        let client = crate::client::get_client();
        let result = client
            .ugc()
            .item_state(pfid(item_id));

        result.bits()
    }

    /// Gets info about currently installed content on the disc for workshop item.
    ///
    /// @returns an object with the the properties {folder, size_on_disk, timestamp}
    ///
    /// {@link https://partner.steamgames.com/doc/api/ISteamUGC#GetItemInstallInfo}
    #[napi]
    pub fn install_info(item_id: BigInt) -> Option<InstallInfo> {
        let client = crate::client::get_client();
        let result = client
            .ugc()
            .item_install_info(pfid(item_id));

        result.map(|install_info| InstallInfo {
            folder: install_info.folder,
            size_on_disk: BigInt::from(install_info.size_on_disk),
            timestamp: install_info.timestamp,
        })
    }

    /// Get info about a pending download of a workshop item.
    ///
    /// @returns an object with the properties {current, total}
    ///
    /// {@link https://partner.steamgames.com/doc/api/ISteamUGC#GetItemDownloadInfo}
    #[napi]
    pub fn download_info(item_id: BigInt) -> Option<DownloadInfo> {
        let client = crate::client::get_client();
        let result = client
            .ugc()
            .item_download_info(pfid(item_id));

        result.map(|download_info| DownloadInfo {
            current: BigInt::from(download_info.0),
            total: BigInt::from(download_info.1),
        })
    }

    /// Download or update a workshop item.
    ///
    /// @param highPriority - If high priority is true, start the download in high priority mode, pausing any existing in-progress Steam downloads and immediately begin downloading this workshop item.
    /// @returns true or false
    ///
    /// {@link https://partner.steamgames.com/doc/api/ISteamUGC#DownloadItem}
    #[napi]
    pub fn download(item_id: BigInt, high_priority: bool) -> bool {
        let client = crate::client::get_client();
        client
            .ugc()
            .download_item(pfid(item_id), high_priority)
    }

    /// Get all subscribed workshop items.
    /// @returns an array of subscribed workshop item ids
    #[napi]
    pub fn get_subscribed_items() -> Vec<BigInt> {
        let client = crate::client::get_client();
        let result = client.ugc().subscribed_items();

        result
            .iter()
            .map(|item| BigInt::from(item.0))
            .collect::<Vec<_>>()
    }

    // Deletes an item
    // @returns true or false
    #[napi]
    pub async fn delete_item(item_id: BigInt) -> Result<(), Error> {
        let client = crate::client::get_client();
        let (tx, rx) = oneshot::channel();
    
        client
            .ugc()
            .delete_item(pfid(item_id), |result| {
                tx.send(result).unwrap();
            });
    
        let result = rx.await.unwrap();
        match result {
            Ok(()) => Ok(()),
            Err(e) => Err(Error::from_reason(e.to_string())),
        }
    }
}
