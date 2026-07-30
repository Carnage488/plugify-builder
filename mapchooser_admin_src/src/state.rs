use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Debug)]
pub enum Screen {
    Home,
    ManualMaps { page: usize },
    ManualAction { map_id: i32, page: usize },
    VoteHome,
    VoteMaps { page: usize },
}

#[derive(Clone, Debug)]
pub struct PendingRoundChange {
    pub map_id: i32,
    pub requested_by: u64,
    pub display_name: String,
    pub source: String,
    pub requested: bool,
}

#[derive(Clone, Debug)]
pub struct ActiveMultiVote {
    pub requested_by: u64,
    pub vote_id: u64,
    pub displaced_next_map_id: i32,
    pub candidates: Vec<i32>,
    pub runoff_started: bool,
}

#[derive(Clone, Debug)]
pub struct ActiveYesNoVote {
    pub map_id: i32,
    pub requested_by: u64,
    pub display_name: String,
    pub displaced_next_map_id: i32,
    pub result: Option<YesNoResult>,
    pub choices: HashMap<i32, i32>,
}

impl ActiveYesNoVote {
    pub fn new(
        map_id: i32,
        requested_by: u64,
        display_name: String,
        displaced_next_map_id: i32,
    ) -> Self {
        Self {
            map_id,
            requested_by,
            display_name,
            displaced_next_map_id,
            result: None,
            choices: HashMap::new(),
        }
    }

    pub fn tracked_result(&self) -> YesNoResult {
        let yes_votes = self.choices.values().filter(|choice| **choice == 0).count() as i32;
        let no_votes = self.choices.values().filter(|choice| **choice == 1).count() as i32;
        YesNoResult {
            passed: yes_votes > no_votes,
            total_votes: yes_votes + no_votes,
            yes_votes,
            no_votes,
        }
    }
}

#[derive(Clone, Debug)]
pub struct YesNoResult {
    pub passed: bool,
    pub total_votes: i32,
    pub yes_votes: i32,
    pub no_votes: i32,
}

#[derive(Clone, Debug)]
pub struct PendingVoteStart {
    pub timer_id: u32,
    pub slot: i32,
    pub requested_by: u64,
    pub candidates: Vec<i32>,
}

fn screens() -> &'static Mutex<HashMap<i32, Screen>> {
    static SCREENS: OnceLock<Mutex<HashMap<i32, Screen>>> = OnceLock::new();
    SCREENS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn selections() -> &'static Mutex<HashMap<i32, Vec<i32>>> {
    static SELECTIONS: OnceLock<Mutex<HashMap<i32, Vec<i32>>>> = OnceLock::new();
    SELECTIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn pending() -> &'static Mutex<Option<PendingRoundChange>> {
    static PENDING: OnceLock<Mutex<Option<PendingRoundChange>>> = OnceLock::new();
    PENDING.get_or_init(|| Mutex::new(None))
}

fn multi_vote() -> &'static Mutex<Option<ActiveMultiVote>> {
    static MULTI_VOTE: OnceLock<Mutex<Option<ActiveMultiVote>>> = OnceLock::new();
    MULTI_VOTE.get_or_init(|| Mutex::new(None))
}

fn yes_no_vote() -> &'static Mutex<Option<ActiveYesNoVote>> {
    static YES_NO_VOTE: OnceLock<Mutex<Option<ActiveYesNoVote>>> = OnceLock::new();
    YES_NO_VOTE.get_or_init(|| Mutex::new(None))
}

fn vote_countdown() -> &'static Mutex<Option<PendingVoteStart>> {
    static VOTE_COUNTDOWN: OnceLock<Mutex<Option<PendingVoteStart>>> = OnceLock::new();
    VOTE_COUNTDOWN.get_or_init(|| Mutex::new(None))
}

pub fn set_screen(slot: i32, screen: Screen) {
    if let Ok(mut guard) = screens().lock() {
        guard.insert(slot, screen);
    }
}

pub fn screen(slot: i32) -> Option<Screen> {
    screens().lock().ok().and_then(|guard| guard.get(&slot).cloned())
}

pub fn clear_screen(slot: i32) {
    if let Ok(mut guard) = screens().lock() {
        guard.remove(&slot);
    }
}

pub fn clear_screens() {
    if let Ok(mut guard) = screens().lock() {
        guard.clear();
    }
}

pub fn selected_maps(slot: i32) -> Vec<i32> {
    selections()
        .lock()
        .ok()
        .and_then(|guard| guard.get(&slot).cloned())
        .unwrap_or_default()
}

pub fn toggle_selected_map(slot: i32, map_id: i32, max_selected: usize) -> ToggleResult {
    let Ok(mut guard) = selections().lock() else {
        return ToggleResult::Failed;
    };
    let selected = guard.entry(slot).or_default();
    if let Some(index) = selected.iter().position(|value| *value == map_id) {
        selected.remove(index);
        return ToggleResult::Removed;
    }
    if selected.len() >= max_selected {
        return ToggleResult::Full;
    }
    selected.push(map_id);
    ToggleResult::Added
}

pub fn retain_selected_maps(slot: i32, allowed: &[i32]) {
    if let Ok(mut guard) = selections().lock() {
        if let Some(selected) = guard.get_mut(&slot) {
            selected.retain(|map_id| allowed.contains(map_id));
        }
    }
}

pub fn clear_selection(slot: i32) {
    if let Ok(mut guard) = selections().lock() {
        guard.remove(&slot);
    }
}

pub fn clear_selections() {
    if let Ok(mut guard) = selections().lock() {
        guard.clear();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToggleResult {
    Added,
    Removed,
    Full,
    Failed,
}

pub fn set_pending(value: PendingRoundChange) {
    if let Ok(mut guard) = pending().lock() {
        *guard = Some(value);
    }
}

pub fn pending_snapshot() -> Option<PendingRoundChange> {
    pending().lock().ok().and_then(|guard| guard.clone())
}

/// Atomically marks the pending map change as requested and returns its snapshot.
/// Returns None when there is no pending change or the request was already sent.
pub fn claim_pending_request() -> Option<PendingRoundChange> {
    let Ok(mut guard) = pending().lock() else {
        return None;
    };
    let pending = guard.as_mut()?;
    if pending.requested {
        return None;
    }
    pending.requested = true;
    Some(pending.clone())
}

pub fn clear_pending() {
    if let Ok(mut guard) = pending().lock() {
        *guard = None;
    }
}

pub fn begin_multi_vote(
    requested_by: u64,
    displaced_next_map_id: i32,
    candidates: Vec<i32>,
) {
    if let Ok(mut guard) = multi_vote().lock() {
        *guard = Some(ActiveMultiVote {
            requested_by,
            vote_id: 0,
            displaced_next_map_id,
            candidates,
            runoff_started: false,
        });
    }
}

pub fn set_multi_vote_id(vote_id: u64) -> bool {
    let Ok(mut guard) = multi_vote().lock() else {
        return false;
    };
    let Some(active) = guard.as_mut() else {
        return false;
    };
    active.vote_id = vote_id;
    true
}

pub fn mark_multi_vote_runoff(vote_id: u64) -> bool {
    let Ok(mut guard) = multi_vote().lock() else {
        return false;
    };
    let Some(active) = guard.as_mut() else {
        return false;
    };
    if active.vote_id != vote_id || vote_id == 0 {
        return false;
    }
    active.runoff_started = true;
    true
}

pub fn is_our_multi_vote(vote_id: u64) -> bool {
    multi_vote()
        .lock()
        .ok()
        .and_then(|guard| guard.as_ref().map(|active| active.vote_id == vote_id && vote_id > 0))
        .unwrap_or(false)
}

pub fn multi_vote_snapshot() -> Option<ActiveMultiVote> {
    multi_vote().lock().ok().and_then(|guard| guard.clone())
}

pub fn clear_multi_vote() {
    if let Ok(mut guard) = multi_vote().lock() {
        *guard = None;
    }
}

pub fn begin_yes_no_vote(value: ActiveYesNoVote) -> bool {
    let Ok(mut guard) = yes_no_vote().lock() else {
        return false;
    };
    if guard.is_some() {
        return false;
    }
    *guard = Some(value);
    true
}

pub fn set_yes_no_result(result: YesNoResult) -> bool {
    let Ok(mut guard) = yes_no_vote().lock() else {
        return false;
    };
    let Some(active) = guard.as_mut() else {
        return false;
    };
    active.result = Some(result);
    true
}

pub fn record_yes_no_choice(slot: i32, choice: i32) -> bool {
    if slot < 0 || !matches!(choice, 0 | 1) {
        return false;
    }
    let Ok(mut guard) = yes_no_vote().lock() else {
        return false;
    };
    let Some(active) = guard.as_mut() else {
        return false;
    };
    active.choices.insert(slot, choice);
    true
}

pub fn yes_no_snapshot() -> Option<ActiveYesNoVote> {
    yes_no_vote().lock().ok().and_then(|guard| guard.clone())
}

pub fn take_yes_no_vote() -> Option<ActiveYesNoVote> {
    yes_no_vote().lock().ok().and_then(|mut guard| guard.take())
}

pub fn clear_yes_no_vote() {
    if let Ok(mut guard) = yes_no_vote().lock() {
        *guard = None;
    }
}

pub fn begin_vote_countdown(value: PendingVoteStart) -> bool {
    let Ok(mut guard) = vote_countdown().lock() else {
        return false;
    };
    if guard.is_some() {
        return false;
    }
    *guard = Some(value);
    true
}

pub fn take_vote_countdown(timer_id: u32) -> Option<PendingVoteStart> {
    let Ok(mut guard) = vote_countdown().lock() else {
        return None;
    };
    if guard.as_ref().is_some_and(|pending| pending.timer_id == timer_id) {
        guard.take()
    } else {
        None
    }
}

pub fn clear_vote_countdown() -> Option<u32> {
    vote_countdown()
        .lock()
        .ok()
        .and_then(|mut guard| guard.take().map(|pending| pending.timer_id))
}

pub fn admin_vote_active() -> bool {
    let countdown_active = vote_countdown()
        .lock()
        .map(|guard| guard.is_some())
        .unwrap_or(false);
    if countdown_active {
        return true;
    }
    let multi_active = multi_vote()
        .lock()
        .map(|guard| guard.is_some())
        .unwrap_or(false);
    if multi_active {
        return true;
    }
    yes_no_vote()
        .lock()
        .map(|guard| guard.is_some())
        .unwrap_or(false)
}

pub fn admin_pending_map_change() -> bool {
    pending()
        .lock()
        .map(|guard| guard.is_some())
        .unwrap_or(false)
}

pub fn clear_runtime() {
    clear_screens();
    clear_selections();
    clear_pending();
    clear_multi_vote();
    clear_yes_no_vote();
    let _ = clear_vote_countdown();
}
