pub unsafe extern "C" fn on_server_check_transmit(infos: &Arr<usize>) {
    let tracked = TRACKED_TARGETS.load(Ordering::Acquire);
    if tracked == 0 {
        return;
    }

    let active_viewers = ACTIVE_VIEWERS.load(Ordering::Acquire);
    let state = entities();

    for info in infos.as_slice().iter().copied() {
        if info == 0 {
            continue;
        }

        let viewer_slot = s2sdk::transmit::GetTransmitInfoPlayerSlot(info);
        let viewer_active = active_viewers & slot_bit(viewer_slot) != 0;

        let mut remaining = tracked;
        while remaining != 0 {
            let target_slot = remaining.trailing_zeros() as i32;
            remaining &= remaining - 1;

            if viewer_active && viewer_slot != target_slot {
                continue;
            }

            let glow = state.glow[target_slot as usize].load(Ordering::Acquire);
            if glow != INVALID_ENTITY_HANDLE {
                s2sdk::transmit::ClearTransmitInfoEntity(info, glow);
            }
        }
    }
}

unsafe extern "C" fn on_esp_command(
    player_slot: i32,
    context: s2sdk::enums::ConCommandContext,
    _args: &Arr<Str>,
) -> s2sdk::enums::ResultType {
    if player_slot < 0 {
        s2sdk::console::ReplyToCommand(
            context,
            player_slot,
            &Str::from(PLAYER_ONLY_MESSAGE),
        );
        return s2sdk::enums::ResultType::Handled;
    }

    toggle(player_slot);
    s2sdk::enums::ResultType::Handled
}

unsafe extern "C" fn on_chat_command(
    player_slot: i32,
    _context: s2sdk::enums::ConCommandContext,
    args: &Arr<Str>,
) -> s2sdk::enums::ResultType {
    let Some(message) = chat_message(args) else {
        return s2sdk::enums::ResultType::Continue;
    };
    if !matches!(message.as_str(), "!esp" | "/esp") {
        return s2sdk::enums::ResultType::Continue;
    }

    toggle(player_slot);
    s2sdk::enums::ResultType::Stop
}

fn toggle(player_slot: i32) {
    if !valid_human_viewer(player_slot) {
        return;
    }

    let bit = slot_bit(player_slot);
    if ENABLED_VIEWERS.load(Ordering::Acquire) & bit != 0 {
        ENABLED_VIEWERS.fetch_and(!bit, Ordering::AcqRel);
        ACTIVE_VIEWERS.fetch_and(!bit, Ordering::AcqRel);
        chat(player_slot, DISABLED_MESSAGE);
        return;
    }

    let Some(config) = crate::config_ref() else {
        chat(player_slot, NO_ACCESS_MESSAGE);
        return;
    };

    let always = rust_admin::check_access(
        player_slot,
        ESP_ALWAYS_ACCESS_COMMAND,
        &config.admin_flag_all,
    )
    .unwrap_or(false);
    let observer_access = rust_admin::check_access(
        player_slot,
        ESP_COMMAND,
        &config.admin_flag_death,
    )
    .unwrap_or(false);

    if !always && !observer_access {
        chat(player_slot, NO_ACCESS_MESSAGE);
        return;
    }
    if !always && !is_dead_or_spectator(player_slot) {
        chat(player_slot, OBSERVER_ONLY_MESSAGE);
        return;
    }

    ENABLED_VIEWERS.fetch_or(bit, Ordering::AcqRel);
    ACTIVE_VIEWERS.fetch_or(bit, Ordering::AcqRel);
    chat(player_slot, ENABLED_MESSAGE);
}

