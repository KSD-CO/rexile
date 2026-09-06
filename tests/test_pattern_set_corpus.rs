use regex::{Regex, RegexSet};

use rexile::PatternSet;

#[path = "../benches/support/pattern_set_workloads.rs"]
mod workloads;

#[test]
fn frozen_acceptance_corpus_has_identical_results() {
    for &family in workloads::FAMILIES {
        for &count in workloads::COUNTS {
            let patterns = workloads::patterns(family, count);
            let set = PatternSet::new(&patterns).unwrap();
            let baseline = RegexSet::new(&patterns).unwrap();
            let regexes: Vec<_> = patterns
                .iter()
                .map(|pattern| Regex::new(pattern).unwrap())
                .collect();
            for &length in workloads::LENGTHS {
                for &density in workloads::DENSITIES {
                    let text = workloads::haystack(family, count, length, density);
                    let context = format!("{family}/{count}/{length}/{density}");
                    assert_eq!(
                        set.matches(&text).iter().collect::<Vec<_>>(),
                        baseline.matches(&text).iter().collect::<Vec<_>>(),
                        "{context}"
                    );
                    let mut expected: Vec<_> = regexes
                        .iter()
                        .enumerate()
                        .flat_map(|(id, re)| {
                            re.captures_iter(&text).map(move |caps| {
                                (
                                    id,
                                    caps.iter()
                                        .map(|cap| cap.map(|cap| (cap.start(), cap.end())))
                                        .collect::<Vec<_>>(),
                                )
                            })
                        })
                        .collect();
                    expected.sort_by_key(|(id, slots)| (slots[0].unwrap().0, *id));
                    let actual: Vec<_> = set
                        .captures_iter(&text)
                        .map(|caps| {
                            (
                                caps.pattern_id(),
                                (0..caps.captures().len())
                                    .map(|i| caps.pos(i))
                                    .collect::<Vec<_>>(),
                            )
                        })
                        .collect();
                    assert_eq!(actual, expected, "{context}");
                    assert_eq!(
                        set.find_iter(&text)
                            .map(|m| (m.pattern_id(), m.start(), m.end()))
                            .collect::<Vec<_>>(),
                        expected
                            .iter()
                            .map(|(id, slots)| {
                                let (s, e) = slots[0].unwrap();
                                (*id, s, e)
                            })
                            .collect::<Vec<_>>(),
                        "{context}"
                    );
                }
            }
        }
    }
}
