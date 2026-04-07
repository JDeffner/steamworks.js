use napi_derive::napi;

#[napi]
pub mod callback {
    use crate::api::FatalTsfn;
    use napi::threadsafe_function::ThreadsafeFunctionCallMode;

    #[napi]
    pub struct Handle {
        handle: Option<steamworks::CallbackHandle>,
    }

    #[napi]
    impl Handle {
        #[napi]
        pub fn disconnect(&mut self) {
            if let Some(handle) = self.handle.take() {
                handle.disconnect();
            }
        }
    }

    #[napi]
    pub enum SteamCallback {
        PersonaStateChange,
        SteamServersConnected,
        SteamServersDisconnected,
        SteamServerConnectFailure,
        LobbyDataUpdate,
        LobbyChatUpdate,
        P2PSessionRequest,
        P2PSessionConnectFail,
        GameLobbyJoinRequested,
        MicroTxnAuthorizationResponse,
    }

    #[napi(ts_generic_types = "C extends keyof import('./callbacks').CallbackReturns")]
    pub fn register(
        #[napi(ts_arg_type = "C")] steam_callback: SteamCallback,
        #[napi(ts_arg_type = "(value: import('./callbacks').CallbackReturns[C]) => void")] handler: FatalTsfn<serde_json::Value>,
    ) -> Handle {
        let threadsafe_handler = handler;

        let handle = match steam_callback {
            SteamCallback::PersonaStateChange => {
                register_callback::<steamworks::PersonaStateChange>(threadsafe_handler)
            }
            SteamCallback::SteamServersConnected => {
                register_callback::<steamworks::SteamServersConnected>(threadsafe_handler)
            }
            SteamCallback::SteamServersDisconnected => {
                register_callback::<steamworks::SteamServersDisconnected>(threadsafe_handler)
            }
            SteamCallback::SteamServerConnectFailure => {
                register_callback::<steamworks::SteamServerConnectFailure>(threadsafe_handler)
            }
            SteamCallback::LobbyDataUpdate => {
                register_callback::<steamworks::LobbyDataUpdate>(threadsafe_handler)
            }
            SteamCallback::LobbyChatUpdate => {
                register_callback::<steamworks::LobbyChatUpdate>(threadsafe_handler)
            }
            SteamCallback::P2PSessionRequest => {
                register_callback::<steamworks::P2PSessionRequest>(threadsafe_handler)
            }
            SteamCallback::P2PSessionConnectFail => {
                register_callback::<steamworks::P2PSessionConnectFail>(threadsafe_handler)
            }
            SteamCallback::GameLobbyJoinRequested => {
                register_callback::<steamworks::GameLobbyJoinRequested>(threadsafe_handler)
            }
            SteamCallback::MicroTxnAuthorizationResponse => {
                register_callback::<steamworks::MicroTxnAuthorizationResponse>(threadsafe_handler)
            }
        };

        Handle {
            handle: Some(handle),
        }
    }

    fn register_callback<C>(
        threadsafe_handler: FatalTsfn<serde_json::Value>,
    ) -> steamworks::CallbackHandle
    where
        C: steamworks::Callback + serde::Serialize,
    {
        let client = crate::client::get_client();
        client.register_callback(move |value: C| {
            let value = serde_json::to_value(&value).unwrap();
            threadsafe_handler.call(value, ThreadsafeFunctionCallMode::Blocking);
        })
    }
}
