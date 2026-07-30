use crate::{config::Config, preferences, rust_admin, s2sdk};
use plugify::{Any, Arr, Str, Var};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicI32, AtomicU64, Ordering};
use std::sync::OnceLock;

const MAX_TRACKED_CLIENTS: usize = 64;
const INVALID_ENTITY_HANDLE: i32 = -1;
const ESP_COMMAND: &str = "rs_esp";
// Virtual access key used only by AdminCheckAccess/group overrides.
// There is no separate console command named rs_esp_always.
const ESP_ALWAYS_ACCESS_COMMAND: &str = "rs_esp_always";

const ENABLED_MESSAGE: &str =
    " {GREEN}[ESP]{DEFAULT} Вы успешно {GREEN}включили{DEFAULT} ESP";
const DISABLED_MESSAGE: &str =
    " {RED}[ESP]{DEFAULT} Вы успешно {RED}выключили{DEFAULT} ESP";
const NO_ACCESS_MESSAGE: &str =
    " {RED}[ESP]{DEFAULT} У вас нет доступа к ESP";
const OBSERVER_ONLY_MESSAGE: &str =
    " {RED}[ESP]{DEFAULT} ESP с вашим флагом доступен только после смерти или в наблюдателях";
const SAVE_ERROR_MESSAGE: &str =
    " {RED}[ESP]{DEFAULT} Не удалось сохранить настройку ESP";
const STEAM_ID_UNAVAILABLE_MESSAGE: &str =
    " {RED}[ESP]{DEFAULT} SteamID64 ещё недоступен, повторите команду через секунду";
const PLAYER_ONLY_MESSAGE: &str =
    "[ESP] Команда доступна только игрокам в игре";

static ENABLED_VIEWERS: AtomicU64 = AtomicU64::new(0);
static ACTIVE_VIEWERS: AtomicU64 = AtomicU64::new(0);
// Slots whose glow entity is fully spawned, linked, and safe for CheckTransmit.
// This mask is published only after the whole pair has been initialized.
static TRACKED_TARGETS: AtomicU64 = AtomicU64::new(0);
static ENTITIES: OnceLock<EntitySlots> = OnceLock::new();

struct EntitySlots {
    relay: [AtomicI32; MAX_TRACKED_CLIENTS],
    glow: [AtomicI32; MAX_TRACKED_CLIENTS],
    pawn: [AtomicI32; MAX_TRACKED_CLIENTS],
    team: [AtomicI32; MAX_TRACKED_CLIENTS],
    model_hash: [AtomicU64; MAX_TRACKED_CLIENTS],
}

impl EntitySlots {
    fn new() -> Self {
        Self {
            relay: std::array::from_fn(|_| AtomicI32::new(INVALID_ENTITY_HANDLE)),
            glow: std::array::from_fn(|_| AtomicI32::new(INVALID_ENTITY_HANDLE)),
            pawn: std::array::from_fn(|_| AtomicI32::new(INVALID_ENTITY_HANDLE)),
            team: std::array::from_fn(|_| AtomicI32::new(0)),
            model_hash: std::array::from_fn(|_| AtomicU64::new(0)),
        }
    }
}

fn entities() -> &'static EntitySlots {
    ENTITIES.get_or_init(EntitySlots::new)
}

pub fn register_commands() -> Result<(), String> {
    let add_command = unsafe { s2sdk::commands::__s2sdk_AddConsoleCommand }
        .ok_or_else(|| "S2SDK AddConsoleCommand is not linked".to_string())?;
    let add_listener = unsafe { s2sdk::commands::__s2sdk_AddCommandListener }
        .ok_or_else(|| "S2SDK AddCommandListener is not linked".to_string())?;

    let command_registered = unsafe {
        add_command(
            &Str::from(ESP_COMMAND),
            &Str::from("Toggle administrator player-outline ESP"),
            s2sdk::enums::ConVarFlag::ClientCanExecute,
            on_esp_command,
            s2sdk::enums::HookMode::Post,
        )
    };
    if !command_registered {
        return Err(format!("failed to register command {ESP_COMMAND}"));
    }

    for name in ["say", "say_team"] {
        let registered = unsafe {
            add_listener(
                &Str::from(name),
                on_chat_command,
                s2sdk::enums::HookMode::Pre,
            )
        };
        if !registered {
            unregister_commands();
            return Err(format!("failed to register command listener {name}"));
        }
    }

    Ok(())
}

pub fn unregister_commands() {
    if let Some(remove_command) = unsafe { s2sdk::commands::__s2sdk_RemoveCommand } {
        unsafe {
            let _ = remove_command(&Str::from(ESP_COMMAND), on_esp_command);
        }
    }

    if let Some(remove_listener) = unsafe { s2sdk::commands::__s2sdk_RemoveCommandListener } {
        for name in ["say", "say_team"] {
            unsafe {
                let _ = remove_listener(
                    &Str::from(name),
                    on_chat_command,
                    s2sdk::enums::HookMode::Pre,
                );
            }
        }
    }
}
