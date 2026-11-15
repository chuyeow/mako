//! Link hint labels: the letters typed to pick one of the page's links without the pointer.

/// The left hand's nine letters first, then the rest of the left hand, so the right hand
/// can stay on the pointer. Never F: it starts hints, and typing it again picks nothing.
pub const ALPHABET: &str = "qweasdzxcrtgvb";

/// `count` labels in the order they are handed out. None is a prefix of another, so each
/// label is complete the moment it is typed. All are as short as possible and within one
/// letter of each other, and the first letters stay single the longest.
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
    fn a_few_links_get_the_left_hand_letters_in_order() {
        assert_eq!(labels(9), ["q", "w", "e", "a", "s", "d", "z", "x", "c"]);
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
    fn the_first_letters_keep_single_labels_longest() {
        let l = labels(20);
        assert_eq!(&l[..9], ["q", "w", "e", "a", "s", "d", "z", "x", "c"]);
        assert_eq!(l.iter().filter(|s| s.len() == 1).count(), 13);
        for n in 0..=420 {
            let singles: String = labels(n).iter().filter(|s| s.len() == 1).map(String::as_str).collect();
            assert!(ALPHABET.starts_with(&singles), "count {n}: {singles}");
        }
    }

    #[test]
    fn no_label_uses_f_the_key_that_starts_hints() {
        assert!(labels(420).iter().all(|l| !l.contains('f')));
    }
}
