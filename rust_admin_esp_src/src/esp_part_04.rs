fn create_glow_pair(
    index: usize,
    pawn_handle: i32,
    model: &str,
    glow_color: String,
) -> Result<(), String> {
    let target_bit = slot_bit(index as i32);

    let glow = s2sdk::entities::CreateEntityByName(&Str::from("prop_dynamic"));
    if glow == INVALID_ENTITY_HANDLE || !s2sdk::entities::IsValidEntHandle(glow) {
        return Err("failed to create glow prop_dynamic".to_string());
    }

    let relay = s2sdk::entities::CreateEntityByName(&Str::from("prop_dynamic"));
    if relay == INVALID_ENTITY_HANDLE || !s2sdk::entities::IsValidEntHandle(relay) {
        detach_and_remove_entity(glow);
        return Err("failed to create relay prop_dynamic".to_string());
    }

    let glow_keys = Arr::<Str>::from(vec![
        "model".to_string(),
        "spawnflags".to_string(),
        "glowcolor".to_string(),
        "glowrange".to_string(),
        "glowrangemin".to_string(),
        "glowteam".to_string(),
        "glowtype".to_string(),
        "glowstate".to_string(),
        "renderamt".to_string(),
    ]);
    let glow_values = Arr::<Var>::from(vec![
        Any::String(model.to_string()),
        Any::Int64(256),
        Any::String(glow_color),
        Any::Int32(5000),
        Any::Int32(-1000),
        Any::Int32(-1),
        Any::Int32(2),
        Any::Int32(3),
        Any::Int32(1),
    ]);
    s2sdk::entities::DispatchSpawn2(glow, &glow_keys, &glow_values);

    let relay_keys = Arr::<Str>::from(vec![
        "model".to_string(),
        "spawnflags".to_string(),
        "rendermode".to_string(),
    ]);
    let relay_values = Arr::<Var>::from(vec![
        Any::String(model.to_string()),
        Any::Int64(256),
        Any::Int32(2),
    ]);
    s2sdk::entities::DispatchSpawn2(relay, &relay_keys, &relay_values);

    if !s2sdk::entities::IsValidEntHandle(glow)
        || !s2sdk::entities::IsValidEntHandle(relay)
    {
        detach_and_remove_entity(glow);
        detach_and_remove_entity(relay);
        return Err("glow pair became invalid during DispatchSpawn2".to_string());
    }

    let follow_value = Var::new(&Any::String("!activator".to_string()));
    s2sdk::entities::AcceptEntityInput(
        relay,
        &Str::from("FollowEntity"),
        pawn_handle,
        pawn_handle,
        &follow_value,
        s2sdk::enums::FieldType::String,
        0,
    );
    s2sdk::entities::AcceptEntityInput(
        glow,
        &Str::from("FollowEntity"),
        relay,
        relay,
        &follow_value,
        s2sdk::enums::FieldType::String,
        0,
    );

    if !s2sdk::entities::IsValidEntHandle(glow)
        || !s2sdk::entities::IsValidEntHandle(relay)
    {
        detach_and_remove_entity(glow);
        detach_and_remove_entity(relay);
        return Err("glow pair became invalid while linking FollowEntity".to_string());
    }

    let state = entities();
    state.glow[index].store(glow, Ordering::Release);
    state.relay[index].store(relay, Ordering::Release);
    TRACKED_TARGETS.fetch_or(target_bit, Ordering::AcqRel);

    Ok(())
}

