pub fn register_events() -> Result<(), String> {
    let hooks = [
        (
            "player_spawn",
            on_player_spawn as s2sdk::delegates::EventCallback,
        ),
        (
            "player_death",
            on_player_death as s2sdk::delegates::EventCallback,
        ),
    ];

    for (name, callback) in hooks {
        let result = s2sdk::events::HookEvent(
            &Str::from(name),
            callback,
            s2sdk::enums::HookMode::Post,
        );
        if result != s2sdk::enums::EventHookError::Okay {
            unregister_events();
            return Err(format!("failed to hook {name}: {result:?}"));
        }
    }

    Ok(())
}

pub fn unregister_events() {
    for (name, callback) in [
        (
            "player_spawn",
            on_player_spawn as s2sdk::delegates::EventCallback,
        ),
        (
            "player_death",
            on_player_death as s2sdk::delegates::EventCallback,
        ),
    ] {
        let _ = s2sdk::events::UnhookEvent(
            &Str::from(name),
            callback,
            s2sdk::enums::HookMode::Post,
        );
    }
}

unsafe extern "C" fn on_player_spawn(
    _name: &Str,
    event: usize,
    _dont_broadcast: bool,
) -> s2sdk::enums::ResultType {
    let slot = s2sdk::events::GetEventPlayerSlot(event, &Str::from("userid"));
    request_remove_target_entities(slot);
    s2sdk::enums::ResultType::Continue
}

unsafe extern "C" fn on_player_death(
    _name: &Str,
    event: usize,
    _dont_broadcast: bool,
) -> s2sdk::enums::ResultType {
    let slot = s2sdk::events::GetEventPlayerSlot(event, &Str::from("userid"));
    request_remove_target_entities(slot);
    s2sdk::enums::ResultType::Continue
}

pub fn recompute_active_viewers(config: &Config) {
    let enabled = ENABLED_VIEWERS.load(Ordering::Acquire);
    if enabled == 0 {
        ACTIVE_VIEWERS.store(0, Ordering::Release);
        return;
    }

    let mut active = 0_u64;
    let max_clients = s2sdk::engine::GetMaxClients().clamp(0, MAX_TRACKED_CLIENTS as i32);
    for slot in 0..max_clients {
        let bit = slot_bit(slot);
        if enabled & bit == 0 {
            continue;
        }

        if !valid_human_viewer(slot) {
            clear_viewer(slot);
            continue;
        }

        let always = rust_admin::check_access(
            slot,
            ESP_ALWAYS_ACCESS_COMMAND,
            &config.admin_flag_all,
        )
        .unwrap_or(false);
        if always {
            active |= bit;
            continue;
        }

        let observer_access = rust_admin::check_access(
            slot,
            ESP_COMMAND,
            &config.admin_flag_death,
        )
        .unwrap_or(false);
        if !observer_access {
            clear_viewer(slot);
            continue;
        }
        if is_dead_or_spectator(slot) {
            active |= bit;
        }
    }

    ACTIVE_VIEWERS.store(active, Ordering::Release);
}

