use napi_derive::napi;

#[napi]
pub mod input {
    use napi::bindgen_prelude::{BigInt, Error};
    use steamworks::sys::EInputActionOrigin;

    /// Converts a raw origin number coming from JS back into the SDK enum.
    ///
    /// `EInputActionOrigin` is `#[repr(u32)]` with contiguous discriminants from
    /// `None` (0) up to and including `Count`, plus the sentinel
    /// `MaximumPossibleValue`. Transmuting any other number would be undefined
    /// behaviour, so unknown values are rejected instead.
    fn action_origin_from_raw(origin: u32) -> Result<EInputActionOrigin, Error> {
        const COUNT: u32 = EInputActionOrigin::k_EInputActionOrigin_Count as u32;
        const MAX: u32 = EInputActionOrigin::k_EInputActionOrigin_MaximumPossibleValue as u32;

        if origin <= COUNT || origin == MAX {
            // SAFETY: the check above guarantees `origin` is one of the enum's
            // declared discriminants.
            Ok(unsafe { std::mem::transmute::<u32, EInputActionOrigin>(origin) })
        } else {
            Err(Error::from_reason(format!(
                "{origin} is not a valid EInputActionOrigin"
            )))
        }
    }

    #[napi(string_enum)]
    pub enum InputType {
        Unknown,
        SteamController,
        XBox360Controller,
        XBoxOneController,
        GenericGamepad,
        PS4Controller,
        AppleMFiController,
        AndroidController,
        SwitchJoyConPair,
        SwitchJoyConSingle,
        SwitchProController,
        MobileTouch,
        PS3Controller,
        PS5Controller,
        SteamDeckController,
    }

    impl From<steamworks::InputType> for InputType {
        fn from(input_type: steamworks::InputType) -> InputType {
            match input_type {
                steamworks::InputType::Unknown => InputType::Unknown,
                steamworks::InputType::SteamController => InputType::SteamController,
                steamworks::InputType::XBox360Controller => InputType::XBox360Controller,
                steamworks::InputType::XBoxOneController => InputType::XBoxOneController,
                steamworks::InputType::GenericGamepad => InputType::GenericGamepad,
                steamworks::InputType::PS4Controller => InputType::PS4Controller,
                steamworks::InputType::AppleMFiController => InputType::AppleMFiController,
                steamworks::InputType::AndroidController => InputType::AndroidController,
                steamworks::InputType::SwitchJoyConPair => InputType::SwitchJoyConPair,
                steamworks::InputType::SwitchJoyConSingle => InputType::SwitchJoyConSingle,
                steamworks::InputType::SwitchProController => InputType::SwitchProController,
                steamworks::InputType::MobileTouch => InputType::MobileTouch,
                steamworks::InputType::PS3Controller => InputType::PS3Controller,
                steamworks::InputType::PS5Controller => InputType::PS5Controller,
                steamworks::InputType::SteamDeckController => InputType::SteamDeckController,
            }
        }
    }

    #[napi]
    pub struct Controller {
        pub(crate) handle: BigInt,
    }

    #[napi]
    impl Controller {
        #[napi]
        pub fn activate_action_set(&self, action_set_handle: BigInt) {
            let client = crate::client::get_client();
            client
                .input()
                .activate_action_set_handle(self.handle.get_u64().1, action_set_handle.get_u64().1)
        }

        #[napi]
        pub fn is_digital_action_pressed(&self, action_handle: BigInt) -> bool {
            let client = crate::client::get_client();
            client
                .input()
                .get_digital_action_data(self.handle.get_u64().1, action_handle.get_u64().1)
                .bState
        }

        #[napi]
        pub fn get_analog_action_vector(&self, action_handle: BigInt) -> AnalogActionVector {
            let client = crate::client::get_client();
            let data = client
                .input()
                .get_analog_action_data(self.handle.get_u64().1, action_handle.get_u64().1);
            AnalogActionVector {
                x: data.x as f64,
                y: data.y as f64,
            }
        }

        #[napi]
        pub fn get_type(&self) -> InputType {
            let client = crate::client::get_client();
            client
                .input()
                .get_input_type_for_handle(self.handle.get_u64().1)
                .into()
        }

        #[napi]
        pub fn get_handle(&self) -> BigInt {
            self.handle.clone()
        }

        /// Get the origin(s) this controller currently binds a digital action to,
        /// within the given action set.
        ///
        /// Each origin is the numeric value of an `EInputActionOrigin`; pass it to
        /// {@link getGlyphForActionOrigin} or {@link getStringForActionOrigin} to
        /// show the player which physical input is bound.
        ///
        /// {@link https://partner.steamgames.com/doc/api/ISteamInput#GetDigitalActionOrigins}
        /// {@link https://partner.steamgames.com/doc/api/ISteamInput#EInputActionOrigin}
        #[napi]
        pub fn get_digital_action_origins(
            &self,
            action_set_handle: BigInt,
            action_handle: BigInt,
        ) -> Vec<u32> {
            let client = crate::client::get_client();
            client
                .input()
                .get_digital_action_origins(
                    self.handle.get_u64().1,
                    action_set_handle.get_u64().1,
                    action_handle.get_u64().1,
                )
                .into_iter()
                .map(|origin| origin as u32)
                .collect()
        }

        /// Get the origin(s) this controller currently binds an analog action to,
        /// within the given action set.
        ///
        /// Each origin is the numeric value of an `EInputActionOrigin`; pass it to
        /// {@link getGlyphForActionOrigin} or {@link getStringForActionOrigin} to
        /// show the player which physical input is bound.
        ///
        /// {@link https://partner.steamgames.com/doc/api/ISteamInput#GetAnalogActionOrigins}
        /// {@link https://partner.steamgames.com/doc/api/ISteamInput#EInputActionOrigin}
        #[napi]
        pub fn get_analog_action_origins(
            &self,
            action_set_handle: BigInt,
            action_handle: BigInt,
        ) -> Vec<u32> {
            let client = crate::client::get_client();
            client
                .input()
                .get_analog_action_origins(
                    self.handle.get_u64().1,
                    action_set_handle.get_u64().1,
                    action_handle.get_u64().1,
                )
                .into_iter()
                .map(|origin| origin as u32)
                .collect()
        }

        /// Open the Steam overlay's binding panel for this controller so the player
        /// can rebind their inputs. Returns false if the overlay is unavailable.
        ///
        /// {@link https://partner.steamgames.com/doc/api/ISteamInput#ShowBindingPanel}
        #[napi]
        pub fn show_binding_panel(&self) -> bool {
            let client = crate::client::get_client();
            client.input().show_binding_panel(self.handle.get_u64().1)
        }
    }

    #[napi(object)]
    pub struct AnalogActionVector {
        pub x: f64,
        pub y: f64,
    }

    #[napi]
    pub fn init() {
        let client = crate::client::get_client();
        client.input().init(false);
    }

    #[napi]
    pub fn get_controllers() -> Vec<Controller> {
        let client = crate::client::get_client();
        client
            .input()
            .get_connected_controllers()
            .into_iter()
            .map(|identity| Controller {
                handle: BigInt::from(identity),
            })
            .collect()
    }

    #[napi]
    pub fn get_action_set(action_set_name: String) -> BigInt {
        let client = crate::client::get_client();
        BigInt::from(client.input().get_action_set_handle(&action_set_name))
    }

    #[napi]
    pub fn get_digital_action(action_name: String) -> BigInt {
        let client = crate::client::get_client();
        BigInt::from(client.input().get_digital_action_handle(&action_name))
    }

    #[napi]
    pub fn get_analog_action(action_name: String) -> BigInt {
        let client = crate::client::get_client();
        BigInt::from(client.input().get_analog_action_handle(&action_name))
    }

    /// Get the local file path of the PNG glyph image for an action origin, as
    /// returned by an action-origin getter such as
    /// {@link Controller.getDigitalActionOrigins}.
    ///
    /// {@link https://partner.steamgames.com/doc/api/ISteamInput#GetGlyphForActionOrigin}
    #[napi]
    pub fn get_glyph_for_action_origin(origin: u32) -> Result<String, Error> {
        let client = crate::client::get_client();
        Ok(client
            .input()
            .get_glyph_for_action_origin(action_origin_from_raw(origin)?))
    }

    /// Get the localized, human readable name of an action origin, such as
    /// "A Button", for showing in on-screen prompts.
    ///
    /// {@link https://partner.steamgames.com/doc/api/ISteamInput#GetStringForActionOrigin}
    #[napi]
    pub fn get_string_for_action_origin(origin: u32) -> Result<String, Error> {
        let client = crate::client::get_client();
        Ok(client
            .input()
            .get_string_for_action_origin(action_origin_from_raw(origin)?))
    }

    /// Load a specific action manifest file from disk instead of the one
    /// configured on the Steamworks partner site. Returns false on failure.
    ///
    /// {@link https://partner.steamgames.com/doc/api/ISteamInput#SetInputActionManifestFilePath}
    #[napi]
    pub fn set_input_action_manifest_file_path(path: String) -> bool {
        let client = crate::client::get_client();
        client.input().set_input_action_manifest_file_path(&path)
    }

    /// Synchronize the API state with the latest Steam Input action data. This is
    /// done automatically while callbacks are running; call it directly right
    /// before reading controller state for the lowest possible latency.
    ///
    /// {@link https://partner.steamgames.com/doc/api/ISteamInput#RunFrame}
    #[napi]
    pub fn run_frame() {
        let client = crate::client::get_client();
        client.input().run_frame()
    }

    #[napi]
    pub fn shutdown() {
        let client = crate::client::get_client();
        client.input().shutdown()
    }
}
