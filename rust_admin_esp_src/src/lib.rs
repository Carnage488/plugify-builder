use plugify::register_plugin;
use std::error::Error;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

mod config;
mod esp;
mod preferences;
mod rust_admin;
mod s2sdk;

static STARTED: AtomicBool = AtomicBool::new(false);
static COMMANDS_REGISTERED: AtomicBool = AtomicBool::new(false);
static TRANSMIT_REGISTERED: AtomicBool = AtomicBool::new(false);
static EVENTS_REGISTERED: AtomicBool = AtomicBool::new(false);
static DISCONNECT_REGISTERED: AtomicBool = AtomicBool::new(false);
static MAP_END_REGISTERED: AtomicBool = AtomicBool::new(false);
static CONFIG: OnceLock<config::Config> = OnceLock::new();
static TIMING: OnceLock<Mutex<Timing>> = OnceLock::new();

struct Timing {
    access_elapsed: f32,
    entities_elapsed: f32,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            access_elapsed: 0.0,
            entities_elapsed: 0.0,
        }
    }
}

pub(crate) fn config_ref() -> Option<&'static config::Config> {
    CONFIG.get()
}

fn on_plugin_start() -> Result<(), Box<dyn Error>> {
    if STARTED.swap(true, Ordering::AcqRel) {
        return Ok(());
    }

    if let Err(error) = initialize() {
        shutdown();
        STARTED.store(false, Ordering::Release);
        return Err(error);
    }

    let config = CONFIG.get().expect("config initialized");
    println!(
        "[RustAdminESP] v0.2.1 loaded; observer_flag={} always_flag={} CT={} T={}",
        config.admin_flag_death,
        config.admin_flag_all,
        config.ct_color.entity_value(),
        config.t_color.entity_value(),
    );
    Ok(())
}

fn initialize() -> Result<(), Box<dyn Error>> {
    ensure_dependencies_linked().map_err(boxed_error)?;
    let config = config::load().map_err(boxed_error)?;
    CONFIG
        .set(config)
        .map_err(|_| boxed_error("RustAdminESP config was already initialized"))?;

    let saved_count = preferences::initialize().map_err(boxed_error)?;
    println!(
        "[RustAdminESP] loaded {saved_count} persistent ESP preference(s) from configs/rust_admin_esp_users.toml"
    );

    esp::register_commands().map_err(boxed_error)?;
    COMMANDS_REGISTERED.store(true, Ordering::Release);

    esp::register_events().map_err(boxed_error)?;
    EVENTS_REGISTERED.store(true, Ordering::Release);

    s2sdk::listeners::OnServerCheckTransmit_Register(esp::on_server_check_transmit);
    TRANSMIT_REGISTERED.store(true, Ordering::Release);

    s2sdk::listeners::OnClientDisconnect_Register(on_client_disconnect);
    DISCONNECT_REGISTERED.store(true, Ordering::Release);

    s2sdk::listeners::OnMapEnd_Register(on_map_end);
    MAP_END_REGISTERED.store(true, Ordering::Release);

    Ok(())
}

fn on_plugin_update(dt: f32) -> Result<(), Box<dyn Error>> {
    if !STARTED.load(Ordering::Acquire) {
        return Ok(());
    }

    let Some(config) = config_ref() else {
        return Ok(());
    };
    let timing = TIMING.get_or_init(|| Mutex::new(Timing::default()));
    let Ok(mut timing) = timing.lock() else {
        return Ok(());
    };

    let dt = dt.clamp(0.0, 1.0);
    timing.access_elapsed += dt;
    timing.entities_elapsed += dt;

    if timing.access_elapsed >= 0.10 {
        timing.access_elapsed = 0.0;
        esp::recompute_active_viewers(config);
    }
    if timing.entities_elapsed >= 0.25 {
        timing.entities_elapsed = 0.0;
        esp::reconcile_entities(config);
    }

    Ok(())
}

fn on_plugin_end() -> Result<(), Box<dyn Error>> {
    if STARTED.swap(false, Ordering::AcqRel) {
        shutdown();
    }
    println!("[RustAdminESP] unloaded");
    Ok(())
}

unsafe extern "C" fn on_client_disconnect(slot: i32) {
    esp::clear_client(slot);
}

unsafe extern "C" fn on_map_end() {
    esp::reset_all();
}

fn shutdown() {
    esp::reset_all();

    if TRANSMIT_REGISTERED.swap(false, Ordering::AcqRel) {
        if let Some(unregister) = unsafe {
            s2sdk::listeners::__s2sdk_OnServerCheckTransmit_Unregister
        } {
            unsafe { unregister(esp::on_server_check_transmit) };
        }
    }

    if EVENTS_REGISTERED.swap(false, Ordering::AcqRel) {
        esp::unregister_events();
    }

    if DISCONNECT_REGISTERED.swap(false, Ordering::AcqRel) {
        if let Some(unregister) = unsafe {
            s2sdk::listeners::__s2sdk_OnClientDisconnect_Unregister
        } {
            unsafe { unregister(on_client_disconnect) };
        }
    }

    if MAP_END_REGISTERED.swap(false, Ordering::AcqRel) {
        if let Some(unregister) = unsafe { s2sdk::listeners::__s2sdk_OnMapEnd_Unregister } {
            unsafe { unregister(on_map_end) };
        }
    }

    if COMMANDS_REGISTERED.swap(false, Ordering::AcqRel) {
        esp::unregister_commands();
    }
}

fn ensure_dependencies_linked() -> Result<(), String> {
    if !rust_admin::linked() {
        return Err("RustAdmin AdminCheckAccess binding is unavailable".to_string());
    }

    let ready = unsafe {
        s2sdk::commands::__s2sdk_AddConsoleCommand.is_some()
            && s2sdk::commands::__s2sdk_RemoveCommand.is_some()
            && s2sdk::commands::__s2sdk_AddCommandListener.is_some()
            && s2sdk::commands::__s2sdk_RemoveCommandListener.is_some()
            && s2sdk::listeners::__s2sdk_OnServerCheckTransmit_Register.is_some()
            && s2sdk::listeners::__s2sdk_OnServerCheckTransmit_Unregister.is_some()
            && s2sdk::listeners::__s2sdk_OnClientDisconnect_Register.is_some()
            && s2sdk::listeners::__s2sdk_OnClientDisconnect_Unregister.is_some()
            && s2sdk::listeners::__s2sdk_OnMapEnd_Register.is_some()
            && s2sdk::listeners::__s2sdk_OnMapEnd_Unregister.is_some()
            && s2sdk::clients::__s2sdk_IsClientInGame.is_some()
            && s2sdk::clients::__s2sdk_IsClientAlive.is_some()
            && s2sdk::clients::__s2sdk_IsFakeClient.is_some()
            && s2sdk::clients::__s2sdk_GetClientTeam.is_some()
            && s2sdk::clients::__s2sdk_GetClientPawn.is_some()
            && s2sdk::clients::__s2sdk_GetClientSteamID64.is_some()
            && s2sdk::engine::__s2sdk_GetMaxClients.is_some()
            && s2sdk::entities::__s2sdk_EntPointerToEntHandle.is_some()
            && s2sdk::entities::__s2sdk_IsValidEntPointer.is_some()
            && s2sdk::entities::__s2sdk_IsValidEntHandle.is_some()
            && s2sdk::entities::__s2sdk_CreateEntityByName.is_some()
            && s2sdk::entities::__s2sdk_DispatchSpawn2.is_some()
            && s2sdk::entities::__s2sdk_RemoveEntity.is_some()
            && s2sdk::entities::__s2sdk_GetEntityModel.is_some()
            && s2sdk::entities::__s2sdk_AcceptEntityInput.is_some()
            && s2sdk::events::__s2sdk_HookEvent.is_some()
            && s2sdk::events::__s2sdk_UnhookEvent.is_some()
            && s2sdk::events::__s2sdk_GetEventPlayerSlot.is_some()
            && s2sdk::transmit::__s2sdk_GetTransmitInfoPlayerSlot.is_some()
            && s2sdk::transmit::__s2sdk_ClearTransmitInfoEntity.is_some()
            && s2sdk::console::__s2sdk_PrintToChatColored.is_some()
            && s2sdk::console::__s2sdk_ReplyToCommand.is_some()
    };
    if !ready {
        return Err("S2SDK bindings required by RustAdminESP are incomplete".to_string());
    }

    Ok(())
}

fn boxed_error(message: impl Into<String>) -> Box<dyn Error> {
    Box::new(io::Error::other(message.into()))
}

register_plugin!(
    start: on_plugin_start,
    update: on_plugin_update,
    end: on_plugin_end
);
