use napi_derive::napi;

#[napi]
pub mod stats {
    #[napi]
    pub fn get_int(name: String) -> Option<i32> {
        let client = crate::client::get_client();
        client.user_stats().get_stat_i32(&name).ok()
    }

    #[napi]
    pub fn set_int(name: String, value: i32) -> bool {
        let client = crate::client::get_client();
        client.user_stats().set_stat_i32(&name, value).is_ok()
    }

    #[napi]
    pub fn get_float(name: String) -> Option<f64> {
        let client = crate::client::get_client();
        client
            .user_stats()
            .get_stat_f32(&name)
            .ok()
            .map(|value| value as f64)
    }

    #[napi]
    pub fn set_float(name: String, value: f64) -> bool {
        let client = crate::client::get_client();
        client
            .user_stats()
            .set_stat_f32(&name, value as f32)
            .is_ok()
    }

    #[napi]
    pub fn store() -> bool {
        let client = crate::client::get_client();
        client.user_stats().store_stats().is_ok()
    }

    #[napi]
    pub fn reset_all(achievements_too: bool) -> bool {
        let client = crate::client::get_client();
        client
            .user_stats()
            .reset_all_stats(achievements_too)
            .is_ok()
    }
}
