#![allow(dead_code, non_snake_case, non_upper_case_globals, non_camel_case_types, static_mut_refs)]

use plugify::{Arr, Str};

pub const API_VERSION: i32 = 1;
pub const REASON_PROBE: i32 = 0;
pub const REASON_ADMIN: i32 = 3;
pub const START_IMMEDIATE: i32 = 0;
pub const STATUS_OK: i32 = 0;
pub const INVALID_MAP_ID: i32 = -1;

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

binding!(MC_GetApiVersion, __mapchooser_core_MC_GetApiVersion, _MC_GetApiVersion, () -> i32, 0);
binding!(MC_GetEligibleMapIds, __mapchooser_core_MC_GetEligibleMapIds, _MC_GetEligibleMapIds, (player_count: i32, reason: i32, requester_steam_id: u64) -> Arr<i32>, Arr::from(Vec::<i32>::new()));
binding!(MC_IsMapEligible, __mapchooser_core_MC_IsMapEligible, _MC_IsMapEligible, (map_id: i32, player_count: i32, reason: i32, requester_steam_id: u64) -> bool, false);
binding!(MC_GetMapDisplayName, __mapchooser_core_MC_GetMapDisplayName, _MC_GetMapDisplayName, (map_id: i32) -> Str, Str::from(""));
binding!(MC_GetMapEngineName, __mapchooser_core_MC_GetMapEngineName, _MC_GetMapEngineName, (map_id: i32) -> Str, Str::from(""));
binding!(MC_GetNextMapId, __mapchooser_core_MC_GetNextMapId, _MC_GetNextMapId, () -> i32, INVALID_MAP_ID);
binding!(MC_SetNextMap, __mapchooser_core_MC_SetNextMap, _MC_SetNextMap, (map_id: i32, requested_by: u64, source: &Str) -> i32, -1000);
binding!(MC_ClearNextMap, __mapchooser_core_MC_ClearNextMap, _MC_ClearNextMap, (requested_by: u64, source: &Str) -> bool, false);
binding!(MC_QueueVote, __mapchooser_core_MC_QueueVote, _MC_QueueVote, (reason: i32, start_mode: i32, requested_by: u64, source: &Str, candidate_map_ids: &Arr<i32>) -> i32, -1000);
binding!(MC_RequestMapChange, __mapchooser_core_MC_RequestMapChange, _MC_RequestMapChange, (map_id: i32, delay_seconds: f32, requested_by: u64, source: &Str) -> i32, -1000);

pub fn linked() -> bool {
    unsafe {
        __mapchooser_core_MC_GetApiVersion.is_some()
            && __mapchooser_core_MC_GetEligibleMapIds.is_some()
            && __mapchooser_core_MC_IsMapEligible.is_some()
            && __mapchooser_core_MC_GetMapDisplayName.is_some()
            && __mapchooser_core_MC_GetMapEngineName.is_some()
            && __mapchooser_core_MC_GetNextMapId.is_some()
            && __mapchooser_core_MC_SetNextMap.is_some()
            && __mapchooser_core_MC_ClearNextMap.is_some()
            && __mapchooser_core_MC_QueueVote.is_some()
            && __mapchooser_core_MC_RequestMapChange.is_some()
    }
}
