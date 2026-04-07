use napi::threadsafe_function::{ThreadsafeFunction, UnknownReturnValue};
use napi::Status;

pub type FatalTsfn<T> = ThreadsafeFunction<T, UnknownReturnValue, T, Status, false>;

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
