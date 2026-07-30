#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]

pub mod enums {
    #[repr(i32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum CSTeam { None = 0, Spectator = 1, T = 2, CT = 3 }

    #[repr(i32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FieldType { Auto = 0, Float32 = 1, Float64 = 2, Int32 = 3, UInt32 = 4, Int64 = 5, UInt64 = 6, Boolean = 7, Character = 8, String = 9, CString = 10, HScript = 11, EHandle = 12, Resource = 13, Vector3d = 14, Vector2d = 15, Vector4d = 16, Color32 = 17, QAngle = 18, Quaternion = 19 }

    #[repr(i64)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ConVarFlag { None = 0, ClientCanExecute = 33_554_432 }

    #[repr(i32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ResultType { Continue = 0, Changed = 1, Handled = 2, Stop = 3 }

    #[repr(i32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ConCommandContext { Console = 0, Chat = 1 }

    #[repr(u8)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum HookMode { Pre = 0, Post = 1 }

    #[repr(i32)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum EventHookError { Okay = 0, InvalidEvent = 1, NotActive = 2, InvalidCallback = 3 }
}

pub mod delegates {
    use super::enums::*;
    use plugify::{Arr, Str};
    pub type ConCommandCallback = unsafe extern "C" fn(i32, ConCommandContext, &Arr<Str>) -> ResultType;
    pub type EventCallback = unsafe extern "C" fn(&Str, usize, bool) -> ResultType;
    pub type OnClientDisconnectCallback = unsafe extern "C" fn(i32);
    pub type OnServerCheckTransmitCallback = unsafe extern "C" fn(&Arr<usize>);
    pub type OnMapEndCallback = unsafe extern "C" fn();
}

pub mod commands {
    use super::{delegates::*, enums::*};
    use plugify::Str;
    pub type _AddConsoleCommand = unsafe extern "C" fn(&Str, &Str, ConVarFlag, ConCommandCallback, HookMode) -> bool;
    #[unsafe(no_mangle)] pub static mut __s2sdk_AddConsoleCommand: Option<_AddConsoleCommand> = None;
    pub type _RemoveCommand = unsafe extern "C" fn(&Str, ConCommandCallback) -> bool;
    #[unsafe(no_mangle)] pub static mut __s2sdk_RemoveCommand: Option<_RemoveCommand> = None;
    pub type _AddCommandListener = unsafe extern "C" fn(&Str, ConCommandCallback, HookMode) -> bool;
    #[unsafe(no_mangle)] pub static mut __s2sdk_AddCommandListener: Option<_AddCommandListener> = None;
    pub type _RemoveCommandListener = unsafe extern "C" fn(&Str, ConCommandCallback, HookMode) -> bool;
    #[unsafe(no_mangle)] pub static mut __s2sdk_RemoveCommandListener: Option<_RemoveCommandListener> = None;
}

pub mod console {
    use super::enums::*;
    use plugify::Str;
    pub type _PrintToChatColored = unsafe extern "C" fn(i32, &Str);
    #[unsafe(no_mangle)] pub static mut __s2sdk_PrintToChatColored: Option<_PrintToChatColored> = None;
    pub fn PrintToChatColored(slot: i32, message: &Str) { unsafe { __s2sdk_PrintToChatColored.expect("PrintToChatColored missing")(slot, message) } }
    pub type _ReplyToCommand = unsafe extern "C" fn(ConCommandContext, i32, &Str);
    #[unsafe(no_mangle)] pub static mut __s2sdk_ReplyToCommand: Option<_ReplyToCommand> = None;
    pub fn ReplyToCommand(context: ConCommandContext, slot: i32, message: &Str) { unsafe { __s2sdk_ReplyToCommand.expect("ReplyToCommand missing")(context, slot, message) } }
}

pub mod listeners {
    use super::delegates::*;
    macro_rules! binding { ($n:ident, $t:ty) => { #[unsafe(no_mangle)] pub static mut $n: Option<$t> = None; }; }
    pub type _OnServerCheckTransmit_Register = unsafe extern "C" fn(OnServerCheckTransmitCallback);
    binding!(__s2sdk_OnServerCheckTransmit_Register, _OnServerCheckTransmit_Register);
    pub fn OnServerCheckTransmit_Register(cb: OnServerCheckTransmitCallback) { unsafe { __s2sdk_OnServerCheckTransmit_Register.expect("OnServerCheckTransmit_Register missing")(cb) } }
    pub type _OnServerCheckTransmit_Unregister = unsafe extern "C" fn(OnServerCheckTransmitCallback);
    binding!(__s2sdk_OnServerCheckTransmit_Unregister, _OnServerCheckTransmit_Unregister);
    pub type _OnClientDisconnect_Register = unsafe extern "C" fn(OnClientDisconnectCallback);
    binding!(__s2sdk_OnClientDisconnect_Register, _OnClientDisconnect_Register);
    pub fn OnClientDisconnect_Register(cb: OnClientDisconnectCallback) { unsafe { __s2sdk_OnClientDisconnect_Register.expect("OnClientDisconnect_Register missing")(cb) } }
    pub type _OnClientDisconnect_Unregister = unsafe extern "C" fn(OnClientDisconnectCallback);
    binding!(__s2sdk_OnClientDisconnect_Unregister, _OnClientDisconnect_Unregister);
    pub type _OnMapEnd_Register = unsafe extern "C" fn(OnMapEndCallback);
    binding!(__s2sdk_OnMapEnd_Register, _OnMapEnd_Register);
    pub fn OnMapEnd_Register(cb: OnMapEndCallback) { unsafe { __s2sdk_OnMapEnd_Register.expect("OnMapEnd_Register missing")(cb) } }
    pub type _OnMapEnd_Unregister = unsafe extern "C" fn(OnMapEndCallback);
    binding!(__s2sdk_OnMapEnd_Unregister, _OnMapEnd_Unregister);
}

pub mod clients {
    use super::enums::CSTeam;
    macro_rules! binding { ($n:ident, $t:ty) => { #[unsafe(no_mangle)] pub static mut $n: Option<$t> = None; }; }
    pub type _IsClientInGame = unsafe extern "C" fn(i32) -> bool; binding!(__s2sdk_IsClientInGame, _IsClientInGame);
    pub fn IsClientInGame(s:i32)->bool{unsafe{__s2sdk_IsClientInGame.expect("IsClientInGame missing")(s)}}
    pub type _IsClientAlive = unsafe extern "C" fn(i32) -> bool; binding!(__s2sdk_IsClientAlive, _IsClientAlive);
    pub fn IsClientAlive(s:i32)->bool{unsafe{__s2sdk_IsClientAlive.expect("IsClientAlive missing")(s)}}
    pub type _IsFakeClient = unsafe extern "C" fn(i32) -> bool; binding!(__s2sdk_IsFakeClient, _IsFakeClient);
    pub fn IsFakeClient(s:i32)->bool{unsafe{__s2sdk_IsFakeClient.expect("IsFakeClient missing")(s)}}
    pub type _GetClientTeam = unsafe extern "C" fn(i32) -> CSTeam; binding!(__s2sdk_GetClientTeam, _GetClientTeam);
    pub fn GetClientTeam(s:i32)->CSTeam{unsafe{__s2sdk_GetClientTeam.expect("GetClientTeam missing")(s)}}
    pub type _GetClientPawn = unsafe extern "C" fn(i32) -> usize; binding!(__s2sdk_GetClientPawn, _GetClientPawn);
    pub fn GetClientPawn(s:i32)->usize{unsafe{__s2sdk_GetClientPawn.expect("GetClientPawn missing")(s)}}
    pub type _GetClientSteamID64 = unsafe extern "C" fn(i32) -> u64; binding!(__s2sdk_GetClientSteamID64, _GetClientSteamID64);
    pub fn GetClientSteamID64(s:i32)->u64{unsafe{__s2sdk_GetClientSteamID64.expect("GetClientSteamID64 missing")(s)}}
}

pub mod engine {
    pub type _GetMaxClients = unsafe extern "C" fn() -> i32;
    #[unsafe(no_mangle)] pub static mut __s2sdk_GetMaxClients: Option<_GetMaxClients> = None;
    pub fn GetMaxClients()->i32{unsafe{__s2sdk_GetMaxClients.expect("GetMaxClients missing")()}}
}

pub mod entities {
    use super::enums::FieldType;
    use plugify::{Arr, Str, Var};
    macro_rules! binding { ($n:ident, $t:ty) => { #[unsafe(no_mangle)] pub static mut $n: Option<$t> = None; }; }
    pub type _EntPointerToEntHandle=unsafe extern "C" fn(usize)->i32; binding!(__s2sdk_EntPointerToEntHandle,_EntPointerToEntHandle);
    pub fn EntPointerToEntHandle(v:usize)->i32{unsafe{__s2sdk_EntPointerToEntHandle.expect("EntPointerToEntHandle missing")(v)}}
    pub type _IsValidEntPointer=unsafe extern "C" fn(usize)->bool; binding!(__s2sdk_IsValidEntPointer,_IsValidEntPointer);
    pub fn IsValidEntPointer(v:usize)->bool{unsafe{__s2sdk_IsValidEntPointer.expect("IsValidEntPointer missing")(v)}}
    pub type _IsValidEntHandle=unsafe extern "C" fn(i32)->bool; binding!(__s2sdk_IsValidEntHandle,_IsValidEntHandle);
    pub fn IsValidEntHandle(v:i32)->bool{unsafe{__s2sdk_IsValidEntHandle.expect("IsValidEntHandle missing")(v)}}
    pub type _CreateEntityByName=unsafe extern "C" fn(&Str)->i32; binding!(__s2sdk_CreateEntityByName,_CreateEntityByName);
    pub fn CreateEntityByName(v:&Str)->i32{unsafe{__s2sdk_CreateEntityByName.expect("CreateEntityByName missing")(v)}}
    pub type _DispatchSpawn2=unsafe extern "C" fn(i32,&Arr<Str>,&Arr<Var>); binding!(__s2sdk_DispatchSpawn2,_DispatchSpawn2);
    pub fn DispatchSpawn2(h:i32,k:&Arr<Str>,v:&Arr<Var>){unsafe{__s2sdk_DispatchSpawn2.expect("DispatchSpawn2 missing")(h,k,v)}}
    pub type _RemoveEntity=unsafe extern "C" fn(i32); binding!(__s2sdk_RemoveEntity,_RemoveEntity);
    pub fn RemoveEntity(h:i32){unsafe{__s2sdk_RemoveEntity.expect("RemoveEntity missing")(h)}}
    pub type _GetEntityModel=unsafe extern "C" fn(i32)->Str; binding!(__s2sdk_GetEntityModel,_GetEntityModel);
    pub fn GetEntityModel(h:i32)->Str{unsafe{__s2sdk_GetEntityModel.expect("GetEntityModel missing")(h)}}
    pub type _AcceptEntityInput=unsafe extern "C" fn(i32,&Str,i32,i32,&Var,FieldType,i32); binding!(__s2sdk_AcceptEntityInput,_AcceptEntityInput);
    pub fn AcceptEntityInput(h:i32,n:&Str,a:i32,c:i32,v:&Var,t:FieldType,o:i32){unsafe{__s2sdk_AcceptEntityInput.expect("AcceptEntityInput missing")(h,n,a,c,v,t,o)}}
}

pub mod events {
    use super::{delegates::EventCallback,enums::{EventHookError,HookMode}};
    use plugify::Str;
    macro_rules! binding { ($n:ident, $t:ty) => { #[unsafe(no_mangle)] pub static mut $n: Option<$t> = None; }; }
    pub type _HookEvent=unsafe extern "C" fn(&Str,EventCallback,HookMode)->EventHookError; binding!(__s2sdk_HookEvent,_HookEvent);
    pub fn HookEvent(n:&Str,c:EventCallback,m:HookMode)->EventHookError{unsafe{__s2sdk_HookEvent.expect("HookEvent missing")(n,c,m)}}
    pub type _UnhookEvent=unsafe extern "C" fn(&Str,EventCallback,HookMode)->EventHookError; binding!(__s2sdk_UnhookEvent,_UnhookEvent);
    pub fn UnhookEvent(n:&Str,c:EventCallback,m:HookMode)->EventHookError{unsafe{__s2sdk_UnhookEvent.expect("UnhookEvent missing")(n,c,m)}}
    pub type _GetEventPlayerSlot=unsafe extern "C" fn(usize,&Str)->i32; binding!(__s2sdk_GetEventPlayerSlot,_GetEventPlayerSlot);
    pub fn GetEventPlayerSlot(e:usize,k:&Str)->i32{unsafe{__s2sdk_GetEventPlayerSlot.expect("GetEventPlayerSlot missing")(e,k)}}
}

pub mod transmit {
    pub type _GetTransmitInfoPlayerSlot=unsafe extern "C" fn(usize)->i32;
    #[unsafe(no_mangle)] pub static mut __s2sdk_GetTransmitInfoPlayerSlot:Option<_GetTransmitInfoPlayerSlot>=None;
    pub fn GetTransmitInfoPlayerSlot(i:usize)->i32{unsafe{__s2sdk_GetTransmitInfoPlayerSlot.expect("GetTransmitInfoPlayerSlot missing")(i)}}
    pub type _ClearTransmitInfoEntity=unsafe extern "C" fn(usize,i32);
    #[unsafe(no_mangle)] pub static mut __s2sdk_ClearTransmitInfoEntity:Option<_ClearTransmitInfoEntity>=None;
    pub fn ClearTransmitInfoEntity(i:usize,h:i32){unsafe{__s2sdk_ClearTransmitInfoEntity.expect("ClearTransmitInfoEntity missing")(i,h)}}
}
