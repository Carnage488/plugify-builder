#![allow(dead_code, non_snake_case, non_upper_case_globals, non_camel_case_types, static_mut_refs)]

use plugify::{Arr, Str};

pub type MenuActionCallback = unsafe extern "C" fn(i32, i32);

pub type _MenuShow = unsafe extern "C" fn(
    i32,
    &Str,
    &Str,
    &Str,
    &Str,
    &Arr<Str>,
    &Arr<Str>,
    &Arr<i32>,
    MenuActionCallback,
) -> bool;
#[unsafe(no_mangle)]
pub static mut __menu_core_MenuShow: Option<_MenuShow> = None;

pub type _MenuClose = unsafe extern "C" fn(i32, &Str) -> bool;
#[unsafe(no_mangle)]
pub static mut __menu_core_MenuClose: Option<_MenuClose> = None;

pub type _MenuCloseOwner = unsafe extern "C" fn(&Str) -> u32;
#[unsafe(no_mangle)]
pub static mut __menu_core_MenuCloseOwner: Option<_MenuCloseOwner> = None;

pub fn show(
    player_slot: i32,
    owner: &Str,
    screen_id: &Str,
    html: &Str,
    title: &Str,
    content_lines: &Arr<Str>,
    option_labels: &Arr<Str>,
    option_actions: &Arr<i32>,
    callback: MenuActionCallback,
) -> bool {
    let Some(function) = (unsafe { __menu_core_MenuShow }) else {
        return false;
    };
    unsafe {
        function(
            player_slot,
            owner,
            screen_id,
            html,
            title,
            content_lines,
            option_labels,
            option_actions,
            callback,
        )
    }
}

pub fn close(player_slot: i32, owner: &Str) -> bool {
    let Some(function) = (unsafe { __menu_core_MenuClose }) else {
        return false;
    };
    unsafe { function(player_slot, owner) }
}

pub fn close_owner(owner: &Str) -> u32 {
    let Some(function) = (unsafe { __menu_core_MenuCloseOwner }) else {
        return 0;
    };
    unsafe { function(owner) }
}

pub fn linked() -> bool {
    unsafe {
        __menu_core_MenuShow.is_some()
            && __menu_core_MenuClose.is_some()
            && __menu_core_MenuCloseOwner.is_some()
    }
}
