use napi_derive::napi;

#[napi]
pub mod achievement {
    #[napi]
    pub fn activate(achievement: String) -> bool {
        let client = crate::client::get_client();
        client
            .user_stats()
            .achievement(&achievement)
            .set()
            .and_then(|_| client.user_stats().store_stats())
            .is_ok()
    }

    #[napi]
    pub fn is_activated(achievement: String) -> bool {
        let client = crate::client::get_client();
        client
            .user_stats()
            .achievement(&achievement)
            .get()
            .unwrap_or(false)
    }

    #[napi]
    pub fn clear(achievement: String) -> bool {
        let client = crate::client::get_client();
        client
            .user_stats()
            .achievement(&achievement)
            .clear()
            .and_then(|_| client.user_stats().store_stats())
            .is_ok()
    }

    #[napi]
    pub fn names() -> Result<Vec<String>, napi::bindgen_prelude::Error> {
        let client = crate::client::get_client();
        client.user_stats().get_achievement_names().ok_or_else(|| {
            napi::bindgen_prelude::Error::from_reason("Failed to get achievement names")
        })
    }
}
