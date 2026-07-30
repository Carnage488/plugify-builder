use plugify::Str;

pub fn linked() -> bool {
    unsafe { __rust_admin_AdminCheckAccess.is_some() }
}

pub fn check_access(player_slot: i32, command: &str, default_flags: &str) -> Option<bool> {
    let function = unsafe { __rust_admin_AdminCheckAccess }?;
    Some(unsafe {
        function(
            player_slot,
            &Str::from(command),
            &Str::from(default_flags),
        )
    })
}

pub type _AdminCheckAccess = unsafe extern "C" fn(i32, &Str, &Str) -> bool;

#[allow(dead_code, non_upper_case_globals)]
#[unsafe(no_mangle)]
pub static mut __rust_admin_AdminCheckAccess: Option<_AdminCheckAccess> = None;
