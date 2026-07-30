use plugify::{Arr, Str};

use crate::state::{Screen, ToggleResult};

const OWNER: &str = "mapchooser_admin";
const ACTION_BACK: i32 = 0;
const ACTION_PREV: i32 = 7;
const ACTION_NEXT: i32 = 8;
const ACTION_EXIT: i32 = 9;

pub unsafe extern "C" fn open_from_admin(slot: i32) {
    if !crate::runtime_ready() {
        crate::chat(slot, "[MapChooser] Управление картами временно недоступно.");
        return;
    }
    if !crate::has_access(slot) {
        crate::chat(slot, "[MapChooser] Недостаточно прав для управления картами.");
        return;
    }
    show_home(slot);
}

unsafe extern "C" fn on_action(slot: i32, action: i32) {
    if !crate::runtime_ready() {
        close(slot);
        crate::chat(slot, "[MapChooser] Управление картами временно недоступно.");
        return;
    }
    if !crate::has_access(slot) {
        close(slot);
        crate::chat(slot, "[MapChooser] Доступ к управлению картами отозван.");
        return;
    }

    if action == ACTION_EXIT {
        close(slot);
        return;
    }

    match crate::state::screen(slot) {
        Some(Screen::Home) => select_home(slot, action),
        Some(Screen::ManualMaps { page }) => select_manual_maps(slot, action, page),
        Some(Screen::ManualAction { map_id, page }) => {
            select_manual_action(slot, action, map_id, page)
        }
        Some(Screen::VoteHome) => select_vote_home(slot, action),
        Some(Screen::VoteMaps { page }) => select_vote_maps(slot, action, page),
        None => show_home(slot),
    }
}

fn select_home(slot: i32, action: i32) {
    match action {
        1 => show_manual_maps(slot, 0),
        2 => show_vote_home(slot),
        ACTION_BACK => {
            close(slot);
            let _ = crate::admin_api::AdminMenuOpen(slot);
        }
        _ => show_home(slot),
    }
}

fn select_manual_maps(slot: i32, action: i32, page: usize) {
    let maps = eligible_maps(slot);
    let items_per_page = crate::config::snapshot().items_per_page;
    let page_count = maps.len().div_ceil(items_per_page).max(1);
    let page = page.min(page_count - 1);

    match action {
        ACTION_BACK => show_home(slot),
        ACTION_PREV if page > 0 => show_manual_maps(slot, page - 1),
        ACTION_NEXT if page + 1 < page_count => show_manual_maps(slot, page + 1),
        digit if (1..=items_per_page as i32).contains(&digit) => {
            let index = page * items_per_page + (digit as usize - 1);
            if let Some((map_id, _)) = maps.get(index) {
                show_manual_action(slot, *map_id, page);
            } else {
                show_manual_maps(slot, page);
            }
        }
        _ => show_manual_maps(slot, page),
    }
}

fn select_manual_action(slot: i32, action: i32, map_id: i32, page: usize) {
    match action {
        ACTION_BACK => show_manual_maps(slot, page),
        1 => crate::change_now(slot, map_id),
        2 => crate::change_after_round(slot, map_id),
        _ => show_manual_action(slot, map_id, page),
    }
}

fn select_vote_home(slot: i32, action: i32) {
    match action {
        ACTION_BACK => show_home(slot),
        1 => show_vote_maps(slot, 0),
        2 => crate::start_selected_vote(slot),
        3 => {
            crate::state::clear_selection(slot);
            show_vote_home(slot);
        }
        _ => show_vote_home(slot),
    }
}

fn select_vote_maps(slot: i32, action: i32, page: usize) {
    let maps = eligible_maps(slot);
    let allowed = maps.iter().map(|(map_id, _)| *map_id).collect::<Vec<_>>();
    crate::state::retain_selected_maps(slot, &allowed);

    let settings = crate::config::snapshot();
    let items_per_page = settings.items_per_page;
    let page_count = maps.len().div_ceil(items_per_page).max(1);
    let page = page.min(page_count - 1);

    match action {
        ACTION_BACK => show_vote_home(slot),
        ACTION_PREV if page > 0 => show_vote_maps(slot, page - 1),
        ACTION_NEXT if page + 1 < page_count => show_vote_maps(slot, page + 1),
        digit if (1..=items_per_page as i32).contains(&digit) => {
            let index = page * items_per_page + (digit as usize - 1);
            if let Some((map_id, _)) = maps.get(index) {
                if crate::state::toggle_selected_map(slot, *map_id, settings.max_vote_maps)
                    == ToggleResult::Full
                {
                    crate::chat(
                        slot,
                        &format!(
                            "[MapChooser] Для голосования можно выбрать не больше {} карт.",
                            settings.max_vote_maps
                        ),
                    );
                }
            }
            show_vote_maps(slot, page);
        }
        _ => show_vote_maps(slot, page),
    }
}

fn show_home(slot: i32) {
    let next_map = crate::next_map_label();
    let html = format!(
        "<b>=== УПРАВЛЕНИЕ КАРТАМИ ===</b><br>\
         <font color='#888'>0 — Назад | 9 — Выход</font><br><br>\
         Следующая карта: <font color='#FFD700'>{}</font><br><br>\
         <font color='#8FD3FF'>!1</font> - Ручная установка<br>\
         <font color='#8FD3FF'>!2</font> - Голосование за карту<br>",
        escape_html(&next_map)
    );
    show_document(
        slot,
        "mapadmin.home",
        "Управление картами",
        &html,
        vec![
            "Ручная установка".to_string(),
            "Голосование за карту".to_string(),
        ],
        vec![
            (1, "Ручная установка".to_string()),
            (2, "Голосование за карту".to_string()),
            (0, "Назад".to_string()),
            (9, "Выход".to_string()),
        ],
        Screen::Home,
    );
}

fn show_manual_maps(slot: i32, requested_page: usize) {
    let maps = eligible_maps(slot);
    let items_per_page = crate::config::snapshot().items_per_page;
    let page_count = maps.len().div_ceil(items_per_page).max(1);
    let page = requested_page.min(page_count - 1);
    let start = page * items_per_page;

    let mut html = format!(
        "<b>=== ВЫБОР КАРТЫ ===</b> <font color='#888'>[{}/{}]</font><br>\
         <font color='#888'>0 — Назад | 7 — Назад страница | 8 — Вперёд | 9 — Выход</font><br><br>",
        page + 1,
        page_count
    );
    let mut content = Vec::new();
    let mut options = Vec::new();

    append_map_page(&maps, start, items_per_page, &[], &mut html, &mut content, &mut options);
    append_navigation(page, page_count, "Назад", &mut options);

    show_document(
        slot,
        "mapadmin.manual.maps",
        "Выбор карты",
        &html,
        content,
        options,
        Screen::ManualMaps { page },
    );
}

fn show_manual_action(slot: i32, map_id: i32, page: usize) {
    let label = map_label(map_id);
    let html = format!(
        "<b>=== РУЧНАЯ УСТАНОВКА ===</b><br>\
         <font color='#888'>0 — Назад | 9 — Выход</font><br><br>\
         Карта: <font color='#FFD700'>{}</font><br><br>\
         <font color='#8FD3FF'>!1</font> - Сейчас<br>\
         <font color='#8FD3FF'>!2</font> - После раунда<br>",
        escape_html(&label)
    );
    show_document(
        slot,
        "mapadmin.manual.action",
        "Когда сменить карту",
        &html,
        vec!["Сейчас".to_string(), "После раунда".to_string()],
        vec![
            (1, "Сейчас".to_string()),
            (2, "После раунда".to_string()),
            (0, "Назад".to_string()),
            (9, "Выход".to_string()),
        ],
        Screen::ManualAction { map_id, page },
    );
}

fn show_vote_home(slot: i32) {
    let settings = crate::config::snapshot();
    let maps = eligible_maps(slot);
    let allowed = maps.iter().map(|(map_id, _)| *map_id).collect::<Vec<_>>();
    crate::state::retain_selected_maps(slot, &allowed);
    let selected = crate::state::selected_maps(slot);

    let selected_text = if selected.is_empty() {
        "нет".to_string()
    } else {
        selected
            .iter()
            .map(|map_id| map_label(*map_id))
            .collect::<Vec<_>>()
            .join(", ")
    };

    let html = format!(
        "<b>=== АДМИНСКОЕ ГОЛОСОВАНИЕ ===</b><br>\
         <font color='#888'>0 — Назад | 9 — Выход</font><br><br>\
         Выбрано: <font color='#FFD700'>{}/{}</font><br>\
         <font color='#AAA'>{}</font><br><br>\
         <font color='#8FD3FF'>!1</font> - Выбрать карты<br>\
         <font color='#8FD3FF'>!2</font> - Запустить голосование<br>\
         <font color='#8FD3FF'>!3</font> - Очистить выбор<br>",
        selected.len(),
        settings.max_vote_maps,
        escape_html(&selected_text)
    );

    show_document(
        slot,
        "mapadmin.vote.home",
        "Админское голосование",
        &html,
        vec![
            "Выбрать карты".to_string(),
            "Запустить голосование".to_string(),
            "Очистить выбор".to_string(),
        ],
        vec![
            (1, "Выбрать карты".to_string()),
            (2, "Запустить голосование".to_string()),
            (3, "Очистить выбор".to_string()),
            (0, "Назад".to_string()),
            (9, "Выход".to_string()),
        ],
        Screen::VoteHome,
    );
}

fn show_vote_maps(slot: i32, requested_page: usize) {
    let settings = crate::config::snapshot();
    let maps = eligible_maps(slot);
    let allowed = maps.iter().map(|(map_id, _)| *map_id).collect::<Vec<_>>();
    crate::state::retain_selected_maps(slot, &allowed);
    let selected = crate::state::selected_maps(slot);

    let items_per_page = settings.items_per_page;
    let page_count = maps.len().div_ceil(items_per_page).max(1);
    let page = requested_page.min(page_count - 1);
    let start = page * items_per_page;

    let mut html = format!(
        "<b>=== ВЫБОР КАРТ ДЛЯ ГОЛОСОВАНИЯ ===</b> <font color='#888'>[{}/{}]</font><br>\
         <font color='#888'>0 — Готово | 7 — Назад страница | 8 — Вперёд | 9 — Выход</font><br>\
         Выбрано: <font color='#FFD700'>{}/{}</font><br><br>",
        page + 1,
        page_count,
        selected.len(),
        settings.max_vote_maps
    );
    let mut content = Vec::new();
    let mut options = Vec::new();

    append_map_page(
        &maps,
        start,
        items_per_page,
        &selected,
        &mut html,
        &mut content,
        &mut options,
    );
    append_navigation(page, page_count, "Готово", &mut options);

    show_document(
        slot,
        "mapadmin.vote.maps",
        "Выбор карт",
        &html,
        content,
        options,
        Screen::VoteMaps { page },
    );
}

fn append_map_page(
    maps: &[(i32, String)],
    start: usize,
    items_per_page: usize,
    selected: &[i32],
    html: &mut String,
    content: &mut Vec<String>,
    options: &mut Vec<(i32, String)>,
) {
    if maps.is_empty() {
        html.push_str("Нет карт, разрешённых текущими фильтрами.<br>");
        content.push("Нет доступных карт".to_string());
        return;
    }

    for (offset, (map_id, label)) in maps.iter().skip(start).take(items_per_page).enumerate() {
        let digit = offset as i32 + 1;
        let marker = if selected.contains(map_id) { "[✓] " } else { "" };
        html.push_str(&format!(
            "<font color='#8FD3FF'>!{digit}</font> - <font color='#FFD700'>{}{}</font><br>",
            marker,
            escape_html(label)
        ));
        let option_label = format!("{marker}{label}");
        content.push(option_label.clone());
        options.push((digit, option_label));
    }
}

fn append_navigation(
    page: usize,
    page_count: usize,
    back_label: &str,
    options: &mut Vec<(i32, String)>,
) {
    if page > 0 {
        options.push((ACTION_PREV, "Предыдущая страница".to_string()));
    }
    if page + 1 < page_count {
        options.push((ACTION_NEXT, "Следующая страница".to_string()));
    }
    options.push((ACTION_BACK, back_label.to_string()));
    options.push((ACTION_EXIT, "Выход".to_string()));
}

fn eligible_maps(slot: i32) -> Vec<(i32, String)> {
    let steam_id = crate::steam_id(slot);
    let player_count = crate::player_count();
    let mut maps = crate::core_api::MC_GetEligibleMapIds(
        player_count,
        crate::core_api::REASON_PROBE,
        steam_id,
    )
    .iter()
    .copied()
    .filter_map(|map_id| {
        let label = map_label(map_id);
        (!label.is_empty()).then_some((map_id, label))
    })
    .collect::<Vec<_>>();
    maps.sort_by(|left, right| left.1.to_lowercase().cmp(&right.1.to_lowercase()));
    maps
}

fn map_label(map_id: i32) -> String {
    let display = crate::core_api::MC_GetMapDisplayName(map_id)
        .as_str()
        .trim()
        .to_string();
    if !display.is_empty() {
        return display;
    }
    crate::core_api::MC_GetMapEngineName(map_id)
        .as_str()
        .trim()
        .to_string()
}

fn show_document(
    slot: i32,
    screen_id: &str,
    title: &str,
    html: &str,
    content: Vec<String>,
    options: Vec<(i32, String)>,
    screen: Screen,
) {
    crate::state::set_screen(slot, screen);
    let content = Arr::from(content.into_iter().map(Str::from).collect::<Vec<_>>());
    let labels = Arr::from(
        options
            .iter()
            .map(|(_, label)| Str::from(label.as_str()))
            .collect::<Vec<_>>(),
    );
    let actions = Arr::from(options.iter().map(|(action, _)| *action).collect::<Vec<_>>());
    if !crate::menu_core::show(
        slot,
        &Str::from(OWNER),
        &Str::from(screen_id),
        &Str::from(html),
        &Str::from(title),
        &content,
        &labels,
        &actions,
        on_action,
    ) {
        crate::chat(slot, "[MapChooser] Не удалось открыть меню.");
        crate::state::clear_screen(slot);
    }
}

pub fn close(slot: i32) {
    crate::state::clear_screen(slot);
    let _ = crate::menu_core::close(slot, &Str::from(OWNER));
}

pub fn close_all() {
    crate::state::clear_screens();
    let _ = crate::menu_core::close_owner(&Str::from(OWNER));
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
