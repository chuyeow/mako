//! Link hint labels: the letters typed to pick one of the page's links without the pointer.

/// Home row first, as in Vimium, so most labels sit under the resting fingers.
pub const ALPHABET: &str = "sadfjklewcmpgh";

/// `count` labels in the order they are handed out. None is a prefix of another, so each
/// label is complete the moment it is typed. All are as short as possible and within one
/// letter of each other, and the home-row letters stay single the longest.
pub fn labels(count: usize) -> Vec<String> {
    let mut labels: Vec<String> = ALPHABET.chars().map(String::from).collect();
    // Grow by turning the least-wanted of the shortest labels into one label per letter.
    while labels.len() < count {
        let shortest = labels[0].len();
        let last = labels.iter().rposition(|l| l.len() == shortest).expect("the first label is shortest");
        let parent = labels.remove(last);
        labels.extend(ALPHABET.chars().map(|c| format!("{parent}{c}")));
    }
    labels.truncate(count);
    labels
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_few_links_get_single_home_row_letters() {
        assert_eq!(labels(4), ["s", "a", "d", "f"]);
        assert!(labels(0).is_empty());
    }

    #[test]
    fn labels_are_distinct_and_none_is_a_prefix_of_another() {
        for n in 0..=420 {
            let l = labels(n);
            assert_eq!(l.len(), n, "count {n}");
            for (i, a) in l.iter().enumerate() {
                assert!(a.chars().all(|c| ALPHABET.contains(c)), "{a}");
                for (j, b) in l.iter().enumerate() {
                    assert!(i == j || !b.starts_with(a.as_str()), "count {n}: {a} is a prefix of {b}");
                }
            }
        }
    }

    #[test]
    fn labels_are_as_short_as_possible() {
        let longest = |n| labels(n).iter().map(String::len).max().unwrap_or(0);
        let shortest = |n| labels(n).iter().map(String::len).min().unwrap_or(0);
        assert_eq!(longest(14), 1);
        assert_eq!(longest(15), 2);
        assert_eq!(longest(196), 2);
        assert_eq!(longest(197), 3);
        for n in 1..=420 { assert!(longest(n) - shortest(n) <= 1, "count {n}"); }
    }

    #[test]
    fn the_home_row_keeps_single_letters_longest() {
        let l = labels(20);
        assert_eq!(&l[..8], ["s", "a", "d", "f", "j", "k", "l", "e"]);
        assert_eq!(l.iter().filter(|s| s.len() == 1).count(), 13);
    }
}
