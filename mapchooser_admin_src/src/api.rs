pub const API_VERSION: i32 = 1;

#[unsafe(no_mangle)]
pub extern "C" fn MCA_GetApiVersion_Exported() -> i32 {
    API_VERSION
}

#[unsafe(no_mangle)]
pub extern "C" fn MCA_IsAdminVoteActive_Exported() -> bool {
    crate::state::admin_vote_active()
}

#[unsafe(no_mangle)]
pub extern "C" fn MCA_HasPendingMapChange_Exported() -> bool {
    crate::state::admin_pending_map_change()
}
