fn valid_target(slot: i32) -> bool {
    s2sdk::clients::IsClientInGame(slot)
        && s2sdk::clients::IsClientAlive(slot)
        && matches!(
            s2sdk::clients::GetClientTeam(slot),
            s2sdk::enums::CSTeam::T | s2sdk::enums::CSTeam::CT
        )
}

fn valid_human_viewer(slot: i32) -> bool {
    slot_index(slot).is_some()
        && s2sdk::clients::IsClientInGame(slot)
        && !s2sdk::clients::IsFakeClient(slot)
}

fn is_dead_or_spectator(slot: i32) -> bool {
    match s2sdk::clients::GetClientTeam(slot) {
        s2sdk::enums::CSTeam::Spectator => true,
        s2sdk::enums::CSTeam::T | s2sdk::enums::CSTeam::CT => {
            !s2sdk::clients::IsClientAlive(slot)
        }
        _ => false,
    }
}

fn valid_pair(index: usize) -> bool {
    let state = entities();
    let relay = state.relay[index].load(Ordering::Acquire);
    let glow = state.glow[index].load(Ordering::Acquire);
    relay != INVALID_ENTITY_HANDLE
        && glow != INVALID_ENTITY_HANDLE
        && s2sdk::entities::IsValidEntHandle(relay)
        && s2sdk::entities::IsValidEntHandle(glow)
}

fn detach_and_remove_entity(entity_handle: i32) {
    if entity_handle == INVALID_ENTITY_HANDLE
        || !s2sdk::entities::IsValidEntHandle(entity_handle)
    {
        return;
    }

    let empty_value = Var::new(&Any::String(String::new()));
    s2sdk::entities::AcceptEntityInput(
        entity_handle,
        &Str::from("FollowEntity"),
        INVALID_ENTITY_HANDLE,
        INVALID_ENTITY_HANDLE,
        &empty_value,
        s2sdk::enums::FieldType::String,
        0,
    );
    s2sdk::entities::RemoveEntity(entity_handle);
}

fn clear_viewer(slot: i32) {
    let Some(_) = slot_index(slot) else {
        return;
    };
    let bit = slot_bit(slot);
    ENABLED_VIEWERS.fetch_and(!bit, Ordering::AcqRel);
    ACTIVE_VIEWERS.fetch_and(!bit, Ordering::AcqRel);
}

fn slot_index(slot: i32) -> Option<usize> {
    (0..MAX_TRACKED_CLIENTS as i32)
        .contains(&slot)
        .then_some(slot as usize)
}

fn slot_bit(slot: i32) -> u64 {
    slot_index(slot).map_or(0, |index| 1_u64 << index)
}

fn hash_model(model: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    model.hash(&mut hasher);
    hasher.finish()
}

fn chat_message(args: &Arr<Str>) -> Option<String> {
    if args.len() < 2 {
        return None;
    }
    let message = args
        .as_slice()
        .iter()
        .skip(1)
        .map(|value| value.as_str())
        .collect::<Vec<_>>()
        .join(" ");
    let message = message.trim().trim_matches('"').trim().to_ascii_lowercase();
    (!message.is_empty()).then_some(message)
}

fn chat(player_slot: i32, message: &str) {
    s2sdk::console::PrintToChatColored(player_slot, &Str::from(message));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_bit_rejects_out_of_range_slots() {
        assert_eq!(slot_bit(-1), 0);
        assert_eq!(slot_bit(64), 0);
        assert_eq!(slot_bit(0), 1);
        assert_eq!(slot_bit(63), 1_u64 << 63);
    }
}
