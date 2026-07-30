pub fn reconcile_entities(config: &Config) {
    if ACTIVE_VIEWERS.load(Ordering::Acquire) == 0 {
        request_remove_all_entities();
        return;
    }

    let max_clients = s2sdk::engine::GetMaxClients().clamp(0, MAX_TRACKED_CLIENTS as i32);
    for slot in 0..MAX_TRACKED_CLIENTS as i32 {
        if slot >= max_clients || !valid_target(slot) {
            request_remove_target_entities(slot);
            continue;
        }

        let pawn_pointer = s2sdk::clients::GetClientPawn(slot);
        if pawn_pointer == 0 || !s2sdk::entities::IsValidEntPointer(pawn_pointer) {
            request_remove_target_entities(slot);
            continue;
        }

        let pawn_handle = s2sdk::entities::EntPointerToEntHandle(pawn_pointer);
        if pawn_handle == INVALID_ENTITY_HANDLE
            || !s2sdk::entities::IsValidEntHandle(pawn_handle)
        {
            request_remove_target_entities(slot);
            continue;
        }

        let team = s2sdk::clients::GetClientTeam(slot);
        let model = s2sdk::entities::GetEntityModel(pawn_handle)
            .as_str()
            .trim()
            .to_string();
        if model.is_empty() {
            request_remove_target_entities(slot);
            continue;
        }

        let index = slot as usize;
        let model_hash = hash_model(&model);
        let team_number = team as i32;
        let state = entities();
        let unchanged = state.pawn[index].load(Ordering::Acquire) == pawn_handle
            && state.team[index].load(Ordering::Acquire) == team_number
            && state.model_hash[index].load(Ordering::Acquire) == model_hash
            && valid_pair(index);
        if unchanged {
            continue;
        }

        if has_published_entities(index) {
            request_remove_target_entities(slot);
            continue;
        }

        let color = match team {
            s2sdk::enums::CSTeam::CT => config.ct_color,
            s2sdk::enums::CSTeam::T => config.t_color,
            _ => continue,
        };

        match create_glow_pair(index, pawn_handle, &model, color.entity_value()) {
            Ok(()) => {
                state.pawn[index].store(pawn_handle, Ordering::Release);
                state.team[index].store(team_number, Ordering::Release);
                state.model_hash[index].store(model_hash, Ordering::Release);
            }
            Err(error) => {
                request_remove_target_entities(slot);
                println!("[RustAdminESP] slot={slot}: {error}");
            }
        }
    }
}

pub fn request_remove_target_entities(slot: i32) {
    let Some(index) = slot_index(slot) else {
        return;
    };
    let bit = slot_bit(slot);
    let state = entities();

    let glow = state.glow[index].load(Ordering::Acquire);
    let relay = state.relay[index].load(Ordering::Acquire);
    if glow == INVALID_ENTITY_HANDLE && relay == INVALID_ENTITY_HANDLE {
        if TRACKED_TARGETS.load(Ordering::Acquire) & bit != 0 {
            clear_target_publication(index, bit);
        }
        return;
    }

    detach_and_remove_entity(glow);
    detach_and_remove_entity(relay);
    clear_target_publication(index, bit);
}

pub fn request_remove_all_entities() {
    for slot in 0..MAX_TRACKED_CLIENTS as i32 {
        request_remove_target_entities(slot);
    }
}

pub fn clear_client(slot: i32) {
    clear_viewer(slot);
    request_remove_target_entities(slot);
}

pub fn reset_all() {
    ENABLED_VIEWERS.store(0, Ordering::Release);
    ACTIVE_VIEWERS.store(0, Ordering::Release);
    request_remove_all_entities();
}

fn clear_target_publication(index: usize, bit: u64) {
    let state = entities();
    state.glow[index].store(INVALID_ENTITY_HANDLE, Ordering::Release);
    state.relay[index].store(INVALID_ENTITY_HANDLE, Ordering::Release);
    state.pawn[index].store(INVALID_ENTITY_HANDLE, Ordering::Release);
    state.team[index].store(0, Ordering::Release);
    state.model_hash[index].store(0, Ordering::Release);
    TRACKED_TARGETS.fetch_and(!bit, Ordering::AcqRel);
}

fn has_published_entities(index: usize) -> bool {
    let state = entities();
    state.relay[index].load(Ordering::Acquire) != INVALID_ENTITY_HANDLE
        || state.glow[index].load(Ordering::Acquire) != INVALID_ENTITY_HANDLE
}

