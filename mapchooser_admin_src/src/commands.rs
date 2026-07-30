use plugify::{Arr, Str};

use crate::s2sdk;

const COMMAND_NAME: &str = "mc_admin_reload";

pub fn register() -> Result<(), String> {
    if s2sdk::commands::AddConsoleCommand(
        &Str::from(COMMAND_NAME),
        &Str::from("Reload MapChooserAdmin configuration"),
        s2sdk::enums::ConVarFlag::GameDll,
        on_reload,
        s2sdk::enums::HookMode::Post,
    ) {
        Ok(())
    } else {
        Err(format!("failed to register command '{COMMAND_NAME}'"))
    }
}

pub fn unregister() {
    let linked = unsafe { s2sdk::commands::__s2sdk_RemoveCommand.is_some() };
    if linked {
        let _ = s2sdk::commands::RemoveCommand(&Str::from(COMMAND_NAME), on_reload);
    }
}

unsafe extern "C" fn on_reload(
    slot: i32,
    context: s2sdk::enums::ConCommandContext,
    _: &Arr<Str>,
) -> s2sdk::enums::ResultType {
    if slot >= 0 {
        reply(slot, context, "Server console/RCON only");
        return s2sdk::enums::ResultType::Handled;
    }

    match crate::reload_config() {
        Ok(()) => reply(slot, context, "MapChooserAdmin configuration reloaded"),
        Err(error) => reply(
            slot,
            context,
            &format!("MapChooserAdmin reload failed: {error}"),
        ),
    }
    s2sdk::enums::ResultType::Handled
}

fn reply(slot: i32, context: s2sdk::enums::ConCommandContext, message: &str) {
    s2sdk::console::ReplyToCommand(context, slot, &Str::from(message));
}
