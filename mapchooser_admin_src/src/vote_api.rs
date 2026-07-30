#![allow(dead_code, non_snake_case, non_upper_case_globals, non_camel_case_types, static_mut_refs)]

use plugify::Str;

pub type VoteEventCallback = unsafe extern "C" fn(i32, u64, i32, u64, &Str);

pub const API_VERSION: i32 = 1;
pub const STATE_IDLE: i32 = 0;
pub const EVENT_STARTED: i32 = 1;
pub const EVENT_RUNOFF_STARTED: i32 = 5;
pub const EVENT_FINISHED: i32 = 6;
pub const EVENT_CANCELLED: i32 = 7;

macro_rules! binding {
    ($name:ident, $slot:ident, $ty:ident, ($($arg:ident : $argty:ty),*) -> $ret:ty, $fallback:expr) => {
        pub fn $name($($arg: $argty),*) -> $ret {
            let Some(function) = (unsafe { $slot }) else {
                return $fallback;
            };
            unsafe { function($($arg),*) }
        }
        pub type $ty = unsafe extern "C" fn($($argty),*) -> $ret;
        #[unsafe(no_mangle)]
        pub static mut $slot: Option<$ty> = None;
    };
}

binding!(MCV_GetApiVersion, __mapchooser_vote_MCV_GetApiVersion, _MCV_GetApiVersion, () -> i32, 0);
binding!(MCV_GetState, __mapchooser_vote_MCV_GetState, _MCV_GetState, () -> i32, -1);
binding!(MCV_GetVoteId, __mapchooser_vote_MCV_GetVoteId, _MCV_GetVoteId, () -> u64, 0);
binding!(MCV_RegisterEventListener, __mapchooser_vote_MCV_RegisterEventListener, _MCV_RegisterEventListener, (owner: &Str, callback: VoteEventCallback) -> bool, false);
binding!(MCV_UnregisterOwner, __mapchooser_vote_MCV_UnregisterOwner, _MCV_UnregisterOwner, (owner: &Str) -> u32, 0);

pub fn linked() -> bool {
    unsafe {
        __mapchooser_vote_MCV_GetApiVersion.is_some()
            && __mapchooser_vote_MCV_GetState.is_some()
            && __mapchooser_vote_MCV_GetVoteId.is_some()
            && __mapchooser_vote_MCV_RegisterEventListener.is_some()
            && __mapchooser_vote_MCV_UnregisterOwner.is_some()
    }
}
