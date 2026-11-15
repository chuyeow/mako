//! The command palette's matching: which menu commands fit what is typed, best first.

use serde::Deserialize;

/// One command as the shell lists it: its menu title and the menus it sits under.
#[derive(Debug, Deserialize)]
pub struct Command {
    pub name: String,
    #[serde(default)]
    pub path: String,
}

/// Indices into `commands` that match `query`, best first; ties keep menu order. Every word
/// typed must start a word of the name or its menu path, or else appear inside the name. An
/// empty query lists every command in menu order.
pub fn rank(query: &str, commands: &[Command]) -> Vec<usize> {
    let typed = words(query);
    let mut found: Vec<(u32, usize)> = commands.iter().enumerate().filter_map(|(i, c)| score(&typed, c).map(|s| (s, i))).collect();
    found.sort_by_key(|&(score, _)| std::cmp::Reverse(score));
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

    fn cmds(list: &[(&str, &str)]) -> Vec<Command> {
        list.iter().map(|(name, path)| Command { name: name.to_string(), path: path.to_string() }).collect()
    }

    fn names(query: &str, commands: &[Command]) -> Vec<String> {
        rank(query, commands).into_iter().map(|i| commands[i].name.clone()).collect()
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
        assert_eq!(rank("", &menu), (0..menu.len()).collect::<Vec<_>>());
        assert_eq!(rank("   ", &menu), (0..menu.len()).collect::<Vec<_>>());
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
}
