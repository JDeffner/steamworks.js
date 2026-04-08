use napi_derive::napi;

#[napi]
pub mod callback {
    use crate::api::FatalTsfn;
    use napi::threadsafe_function::ThreadsafeFunctionCallMode;
    use napi_derive::napi;

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

    macro_rules! steam_callbacks {
        ($($variant:ident),* $(,)?) => {
            #[napi]
            pub enum SteamCallback {
                $($variant),*
            }

            #[napi(ts_generic_types = "C extends keyof import('./callbacks').CallbackReturns")]
            pub fn register(
                #[napi(ts_arg_type = "C")] steam_callback: SteamCallback,
                #[napi(ts_arg_type = "(value: import('./callbacks').CallbackReturns[C]) => void")] handler: FatalTsfn<serde_json::Value>,
            ) -> Handle {
                let threadsafe_handler = handler;

                let handle = match steam_callback {
                    $(
                        SteamCallback::$variant => {
                            register_callback::<steamworks::$variant>(threadsafe_handler)
                        }
                    )*
                };

                Handle {
                    handle: Some(handle),
                }
            }
        };
    }

    steam_callbacks!(
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
    );

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
