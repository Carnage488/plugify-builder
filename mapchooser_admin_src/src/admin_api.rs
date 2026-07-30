#![allow(dead_code, non_snake_case, non_upper_case_globals, non_camel_case_types, static_mut_refs)]

use plugify::Str;

pub type AdminMenuItemCallback = unsafe extern "C" fn(i32);

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

binding!(
    AdminMenuRegisterItem,
    __rust_admin_AdminMenuRegisterItem,
    _AdminMenuRegisterItem,
    (owner: &Str, identity: &Str, label: &Str, order: i32, callback: AdminMenuItemCallback) -> bool,
    false
);

binding!(
    AdminMenuUnregisterOwner,
    __rust_admin_AdminMenuUnregisterOwner,
    _AdminMenuUnregisterOwner,
    (owner: &Str) -> u32,
    0
);

binding!(
    AdminCheckAccess,
    __rust_admin_AdminCheckAccess,
    _AdminCheckAccess,
    (player_slot: i32, permission: &Str, default_flags: &Str) -> bool,
    false
);

binding!(
    AdminMenuOpen,
    __rust_admin_AdminMenuOpen,
    _AdminMenuOpen,
    (player_slot: i32) -> bool,
    false
);

pub fn linked() -> bool {
    unsafe {
        __rust_admin_AdminMenuRegisterItem.is_some()
            && __rust_admin_AdminMenuUnregisterOwner.is_some()
            && __rust_admin_AdminCheckAccess.is_some()
            && __rust_admin_AdminMenuOpen.is_some()
    }
}
