//! The command palette's matching: which menu commands fit what is typed, best first.

use crate::history::frecency;
use crate::Config;
use serde::{Deserialize, Serialize};

/// The most commands whose runs are kept; past it the least used goes.
pub const MAX_USES: usize = 200;

/// One command as the shell lists it: its menu title, the menus it sits under, and what it
/// does (its action), which a title can change around (Hide Sidebar, Show Sidebar). Without
/// an id it is never counted.
#[derive(Debug, Deserialize)]
pub struct Command {
    pub name: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub id: String,
}

/// How often and how recently each command ran from the palette.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Uses {
    pub commands: Vec<Use>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Use {
    pub id: String,
    pub runs: u32,
    /// Unix seconds.
    pub last_run: u64,
}

/// The uses after `id` ran at `now`. Unchanged with no id or with `remember_history = false`.
pub fn record(cfg: &Config, mut uses: Uses, id: &str, now: u64) -> Uses {
    if id.is_empty() || !cfg.remember_history {
        return uses;
    }
    if let Some(found) = uses.commands.iter_mut().find(|u| u.id == id) {
        found.runs = found.runs.saturating_add(1);
        found.last_run = now;
        return uses;
    }
    if uses.commands.len() >= MAX_USES {
        let least = uses.commands.iter().enumerate().min_by(|(_, a), (_, b)| lift(a, now).total_cmp(&lift(b, now))).map(|(i, _)| i);
        if let Some(i) = least {
            uses.commands.remove(i);
        }
    }
    uses.commands.push(Use { id: id.to_string(), runs: 1, last_run: now });
    uses
}

fn lift(u: &Use, now: u64) -> f64 {
    frecency(u.runs, u.last_run, now)
}

/// Indices into `commands` that match `query`, best first; ties keep menu order. Every word
/// typed must start a word of the name or its menu path, or else appear inside the name. How
/// well it matches is then multiplied by one plus the command's frecency, so a command run
/// often or lately rises, and one never run keeps its plain match. An empty query lists every
/// command by frecency alone, the rest in menu order.
pub fn rank(query: &str, commands: &[Command], uses: &Uses, now: u64) -> Vec<usize> {
    let typed = words(query);
    let used = |c: &Command| uses.commands.iter().find(|u| !c.id.is_empty() && u.id == c.id).map_or(0.0, |u| lift(u, now));
    let mut found: Vec<(f64, usize)> = commands
        .iter()
        .enumerate()
        .filter_map(|(i, c)| score(&typed, c).map(|s| (if typed.is_empty() { used(c) } else { f64::from(s) * (1.0 + used(c)) }, i)))
        .collect();
    found.sort_by(|a, b| b.0.total_cmp(&a.0));
    found.into_iter().map(|(_, i)| i).collect()
}

/// How well every typed word fits: the name's first word 4, a later word's start 3, inside
/// the name or the start of a menu's name 1. None when a word fits nowhere.
fn score(typed: &[String], command: &Command) -> Option<u32> {
    let name = words(&command.name);
    let whole = command.name.to_lowercase();
    let path = words(&command.path);
    typed.iter().try_fold(0, |sum, word| {
        let fit = match name.iter().position(|w| w.starts_with(word.as_str())) {
            Some(0) => 4,
            Some(_) => 3,
            None if whole.contains(word.as_str()) || path.iter().any(|w| w.starts_with(word.as_str())) => 1,
            None => return None,
        };
        Some(sum + fit)
    })
}

fn words(text: &str) -> Vec<String> {
    text.to_lowercase().split(|c: char| !c.is_alphanumeric()).filter(|w| !w.is_empty()).map(String::from).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const DAY: u64 = 86_400;
    const NOW: u64 = 1_800_000_000;

    /// Each command's id is its name, so uses can name it.
    fn cmds(list: &[(&str, &str)]) -> Vec<Command> {
        list.iter().map(|(name, path)| Command { name: name.to_string(), path: path.to_string(), id: name.to_string() }).collect()
    }

    fn names(query: &str, commands: &[Command]) -> Vec<String> {
        used(query, commands, &Uses::default())
    }

    fn used(query: &str, commands: &[Command], uses: &Uses) -> Vec<String> {
        rank(query, commands, uses, NOW).into_iter().map(|i| commands[i].name.clone()).collect()
    }

    fn uses(list: &[(&str, u32, u64)]) -> Uses {
        Uses { commands: list.iter().map(|(id, runs, ago)| Use { id: id.to_string(), runs: *runs, last_run: NOW - ago }).collect() }
    }

    fn cfg(remember_history: bool) -> Config {
        Config { remember_history, ..crate::parse_config("").config }
    }

    fn menu() -> Vec<Command> {
        cmds(&[
            ("New Blank Page", "File"),
            ("New Trail", "File"),
            ("New Page in Trail", "File"),
            ("Close Page", "File"),
            ("Reload", "View"),
            ("Zoom In", "View"),
            ("Zoom Out", "View"),
            ("Back", "History"),
            ("Clear History…", "History"),
            ("Rename Page…", "Trails"),
            ("Mako docs", "Trails"),
        ])
    }

    #[test]
    fn an_empty_query_lists_every_command_in_menu_order() {
        let menu = menu();
        assert_eq!(rank("", &menu, &Uses::default(), NOW), (0..menu.len()).collect::<Vec<_>>());
        assert_eq!(rank("   ", &menu, &Uses::default(), NOW), (0..menu.len()).collect::<Vec<_>>());
    }

    #[test]
    fn each_word_typed_matches_the_start_of_a_word_in_any_case() {
        assert_eq!(names("zo in", &menu()), ["Zoom In"]);
        assert_eq!(names("ZOOM", &menu()), ["Zoom In", "Zoom Out"]);
        assert_eq!(names("new tr", &menu()), ["New Trail", "New Page in Trail"]);
    }

    #[test]
    fn every_word_must_match() {
        assert!(names("zoom sideways", &menu()).is_empty());
    }

    #[test]
    fn a_name_starting_with_what_is_typed_comes_first() {
        let list = cmds(&[("Close Page", ""), ("Page Down", "")]);
        assert_eq!(names("page", &list), ["Page Down", "Close Page"]);
        assert_eq!(names("re", &menu()), ["Reload", "Rename Page…"]);
    }

    #[test]
    fn a_word_start_beats_a_match_inside_a_word() {
        let list = cmds(&[("Homepage", ""), ("Close Page", "")]);
        assert_eq!(names("page", &list), ["Close Page", "Homepage"]);
    }

    #[test]
    fn the_menu_path_matches_too_but_ranks_below_the_name() {
        assert_eq!(names("history", &menu()), ["Clear History…", "Back"]);
        assert_eq!(names("trail", &menu()), ["New Trail", "New Page in Trail", "Rename Page…", "Mako docs"]);
        assert_eq!(names("view zoom", &menu()), ["Zoom In", "Zoom Out"]);
    }

    #[test]
    fn punctuation_in_a_name_does_not_get_in_the_way() {
        assert_eq!(names("rename", &menu()), ["Rename Page…"]);
        assert_eq!(names("rename page", &menu()), ["Rename Page…"]);
    }

    #[test]
    fn an_empty_query_lists_the_most_used_first_then_the_rest_in_menu_order() {
        let used_list = used("", &menu(), &uses(&[("Reload", 1, 0), ("Zoom In", 9, DAY)]));
        assert_eq!(used_list[..3], ["Zoom In", "Reload", "New Blank Page"]);
        assert_eq!(used_list.len(), menu().len());
    }

    #[test]
    fn use_lifts_a_weaker_match_over_a_stronger_one() {
        let list = cmds(&[("Page Down", ""), ("Close Page", "")]);
        assert_eq!(used("page", &list, &uses(&[("Close Page", 1, 0)])), ["Close Page", "Page Down"]);
    }

    #[test]
    fn use_breaks_ties() {
        assert_eq!(used("zoom", &menu(), &uses(&[("Zoom Out", 1, DAY)])), ["Zoom Out", "Zoom In"]);
    }

    #[test]
    fn old_use_fades() {
        let list = cmds(&[("Page Down", ""), ("Close Page", "")]);
        assert_eq!(used("page", &list, &uses(&[("Close Page", 1, 120 * DAY)])), ["Page Down", "Close Page"]);
    }

    #[test]
    fn use_never_shows_a_command_that_does_not_match() {
        assert_eq!(used("zoom", &menu(), &uses(&[("Reload", 50, 0)])), ["Zoom In", "Zoom Out"]);
    }

    #[test]
    fn uses_follow_the_id_not_the_title() {
        let list = vec![
            Command { name: "Hide Mako".into(), path: "".into(), id: "hide:".into() },
            Command { name: "Hide Sidebar".into(), path: "".into(), id: "toggleSidebar:".into() },
        ];
        let names: Vec<_> = rank("hide", &list, &uses(&[("toggleSidebar:", 3, 0)]), NOW).into_iter().map(|i| list[i].name.clone()).collect();
        assert_eq!(names, ["Hide Sidebar", "Hide Mako"]);
    }

    #[test]
    fn a_command_without_an_id_is_never_lifted() {
        let list = vec![
            Command { name: "Page one".into(), path: "Trails".into(), id: "".into() },
            Command { name: "Page two".into(), path: "Trails".into(), id: "".into() },
        ];
        assert_eq!(rank("page", &list, &uses(&[("", 9, 0)]), NOW), [0, 1]);
    }

    #[test]
    fn record_counts_runs_and_keeps_the_last_time() {
        let once = record(&cfg(true), Uses::default(), "reload:", NOW - DAY);
        assert_eq!(once, Uses { commands: vec![Use { id: "reload:".into(), runs: 1, last_run: NOW - DAY }] });
        let twice = record(&cfg(true), once, "reload:", NOW);
        assert_eq!(twice.commands, [Use { id: "reload:".into(), runs: 2, last_run: NOW }]);
    }

    #[test]
    fn record_keeps_nothing_with_history_off_or_without_an_id() {
        assert_eq!(record(&cfg(false), Uses::default(), "reload:", NOW), Uses::default());
        assert_eq!(record(&cfg(true), Uses::default(), "", NOW), Uses::default());
    }

    #[test]
    fn record_at_the_cap_drops_the_least_used() {
        let mut full = Uses { commands: (0..MAX_USES).map(|i| Use { id: format!("c{i}:"), runs: 5, last_run: NOW }).collect() };
        full.commands[7] = Use { id: "stale:".into(), runs: 1, last_run: NOW - 90 * DAY };
        let after = record(&cfg(true), full, "new:", NOW);
        assert_eq!(after.commands.len(), MAX_USES);
        assert!(after.commands.iter().any(|u| u.id == "new:"));
        assert!(!after.commands.iter().any(|u| u.id == "stale:"));
    }
}
