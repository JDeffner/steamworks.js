use napi::threadsafe_function::{ThreadsafeFunction, UnknownReturnValue};
use napi::Status;

pub type FatalTsfn<T> = ThreadsafeFunction<T, UnknownReturnValue, T, Status, false>;

macro_rules! impl_enum_from {
    ($src:path => $dst:path { $($variant:ident),* $(,)? }) => {
        impl From<$src> for $dst {
            fn from(value: $src) -> Self {
                use $src as Src;
                use $dst as Dst;
                match value {
                    $(Src::$variant => Dst::$variant,)*
                }
            }
        }
    };
}
pub(crate) use impl_enum_from;

pub mod achievement;
pub mod apps;
pub mod auth;
pub mod callback;
pub mod cloud;
pub mod input;
pub mod localplayer;
pub mod matchmaking;
pub mod networking;
pub mod overlay;
pub mod stats;
pub mod utils;
pub mod workshop;
pub mod workshop_item;
