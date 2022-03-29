use splr::Certificate;
use std::clone::Clone;
use std::collections::HashMap;
use std::iter;
use std::ops::Neg;

/// All the potential propositions in this puzzle.
#[derive(Debug, PartialEq, Eq)]
enum Proposition {
    /// True if `pattern` represents digit `digit`.
    PatternIsDigit { pattern: u8, digit: u8 },

    /// Negation of PatternIsDigit
    PatternIsNotDigit { pattern: u8, digit: u8 },

    /// True if `wire` maps to `segment`.
    WireIsSegment { wire: char, segment: char },

    /// Negation of WireIsSegment
    WireIsNotSegment { wire: char, segment: char },
}

impl Proposition {
    const MAX_INDEX: i32 = 149;

    /// The the index representing a proposition.
    fn to_index(&self) -> i32 {
        match *self {
            Proposition::PatternIsDigit { pattern, digit } => {
                let pattern = pattern as i32;
                let digit = digit as i32;
                10 * pattern + digit + 1
            }
            Proposition::PatternIsNotDigit { pattern, digit } => {
                let pattern = pattern as i32;
                let digit = digit as i32;
                (10 * pattern + digit + 1).neg()
            }
            Proposition::WireIsSegment { wire, segment } => {
                let a = 'a' as i32;
                let wire = (wire as i32) - a;
                let segment = (segment as i32) - a;
                7 * wire + segment + 101
            }
            Proposition::WireIsNotSegment { wire, segment } => {
                let a = 'a' as i32;
                let wire = (wire as i32) - a;
                let segment = (segment as i32) - a;
                (7 * wire + segment + 101).neg()
            }
        }
    }

    // /// The the index representing the negation of a proposition.
    // fn negation_to_index(&self) -> i32 {
    //     self.to_index().neg()
    // }

    /// Converts a positive index back into a proposition
    fn from_index(index: i32) -> Self {
        if index <= 0 {
            panic!("Indices must be positive.");
        } else if index <= 100 {
            Proposition::PatternIsDigit {
                pattern: ((index - 1) / 10) as u8,
                digit: ((index - 1) % 10) as u8,
            }
        } else if index <= Proposition::MAX_INDEX {
            let a = 'a' as i32;
            Proposition::WireIsSegment {
                wire: ((((index - 101) / 7) + a) as u8) as char,
                segment: ((((index - 101) % 7) + a) as u8) as char,
            }
        } else {
            panic!("Index {} is too high.", index);
        }
    }
}

struct Entry {
    /// The clauses of this entry in conjunctive normal form.
    _clauses: Vec<Vec<i32>>,
}

/// One line of the puzzle
impl Entry {
    fn new() -> Self {
        let mut myself = Self {
            _clauses: Vec::new(),
        };

        // Each pattern must represent *exactly* one digit.
        myself.create_bijection(0..10, |pattern, digit| Proposition::PatternIsNotDigit {
            pattern,
            digit,
        });

        // Each wire must represent *exactly* one segment.
        myself.create_bijection('a'..='g', |wire, segment| Proposition::WireIsNotSegment {
            wire,
            segment,
        });

        // Each wire must represent at least one segment.. DELETE THIS
        for wire in 'a'..='g' {
            myself.add_clause(
                &('a'..='g')
                    .map(|segment| Proposition::WireIsSegment { wire, segment })
                    .collect::<Vec<_>>(),
            );
        }

        myself
    }

    fn create_bijection<T, R, F>(&mut self, range: R, to_proposition: F)
    where
        T: PartialEq + Eq + Copy,
        R: Iterator<Item = T> + Clone,
        F: Fn(T, T) -> Proposition,
    {
        // There cannot be two arrows from any element in the domain
        for x in range.clone() {
            for y1 in range.clone() {
                for y2 in range.clone() {
                    if y1 != y2 {
                        self.add_clause(&[to_proposition(x, y1), to_proposition(x, y2)]);
                    }
                }
            }
        }

        // There cannot be two arrows into any element in the domain
        for x1 in range.clone() {
            for x2 in range.clone() {
                if x1 != x2 {
                    for y in range.clone() {
                        self.add_clause(&[to_proposition(x1, y), to_proposition(x2, y)]);
                    }
                }
            }
        }
    }

    /// Adds a clause to this entry
    fn add_clause(&mut self, clause: &[Proposition]) {
        self._clauses
            .push(clause.iter().map(|p| p.to_index()).collect());
    }

    /// Returns a vector of propositions which solves this entry.
    fn solve(self) -> Vec<Proposition> {
        match Certificate::try_from(self._clauses).unwrap() {
            Certificate::UNSAT => {
                panic!("Not satisfied.");
            }
            Certificate::SAT(soln) => soln
                .iter()
                .filter_map(|&index| {
                    if index > 0 {
                        Some(Proposition::from_index(index))
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }
}

/// Which segments are shown for each segment.
const DIGIT_SEGMENTS: [&str; 10] = [
    "abcefg", "cf", "acdeg", "acdfg", "bcdf", "abdfg", "abdefg", "acf", "abcdefg", "abcdfg",
];

/// Sort the characters in a string.
fn sort_chars(s: &str) -> String {
    let mut s: Vec<char> = s.chars().collect();
    s.sort_unstable();
    s.iter().collect()
}

fn main() {
    let digits: Vec<Vec<u8>> = solve_for_digits();
    println!("digits: {digits:?}");

    println!("Puzzle 8a: {} (387)", solve_8a(&digits));
    println!("Puzzle !8b: {} (986034)", solve_8b(&digits));
}

fn solve_8a(digits: &[Vec<u8>]) -> usize {
    digits
        .iter()
        .flatten()
        .filter(|&&x| x == 1 || x == 4 || x == 7 || x == 8)
        .count()
}

fn solve_8b(digits: &[Vec<u8>]) -> usize {
    digits
        .iter()
        .map(|digits| {
            digits
                .iter()
                .fold(0usize, |result, &digit| result * 10 + (digit as usize))
        })
        .sum()
}

fn solve_for_digits() -> Vec<Vec<u8>> {
    let len_to_digits: HashMap<usize, Vec<u8>> = HashMap::from_iter((2..=7).map(|n_segments| {
        (
            n_segments,
            DIGIT_SEGMENTS
                .iter()
                .enumerate()
                .filter_map(|pair| match pair {
                    (i, s) if s.len() == n_segments => Some(i as u8),
                    _ => None,
                })
                .collect(),
        )
    }));

    include_str!("../../puzzle_inputs/day_8.txt")
        .lines()
        .enumerate()
        .map(|(line_no, line)| {
            let (patterns, output) = line.split_once("|").unwrap();
            let patterns: Vec<&str> = patterns.split_whitespace().collect();
            let output: Vec<&str> = output.split_whitespace().collect();
            println!("{line_no} patterns: {patterns:?}");
            println!("{line_no} output: {output:?}");

            // Setup the SAT puzzle.
            let mut entry = Entry::new();
            for (pattern, chars) in patterns.iter().enumerate() {
                let pattern = pattern as u8;
                println!("{pattern} -> {chars}");
                let mut potential_digits = Vec::new();
                for &digit in len_to_digits[&chars.len()].iter() {
                    potential_digits.push(Proposition::PatternIsDigit { pattern, digit });
                    let segments = DIGIT_SEGMENTS[digit as usize];
                    for wire in chars.chars() {
                        entry.add_clause(&Vec::from_iter(
                            segments
                                .chars()
                                .map(|segment| Proposition::WireIsSegment { wire, segment })
                                .chain(iter::once({
                                    Proposition::PatternIsNotDigit { pattern, digit }
                                })),
                        ));
                    }
                }
                entry.add_clause(&potential_digits);
            }

            // Solve the SAT puzzle.
            let soln = entry.solve();
            for prop in soln.iter() {
                println!("{prop:?}");
            }

            // Create a mapping from characters to digits
            let digit_map: HashMap<String, u8> =
                HashMap::from_iter(soln.iter().filter_map(|prop| match prop {
                    Proposition::PatternIsDigit { pattern, digit } => {
                        Some((sort_chars(patterns[*pattern as usize]), *digit))
                    }
                    _ => None,
                }));
            println!("digits: {digit_map:?}");
            let output: Vec<u8> = output
                .iter()
                .map(|chars| digit_map[&sort_chars(chars)])
                .collect();

            // panic!("output_digits: output");
            output
        })
        .collect()
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     /// Test that convert from propositions to indices and back works.
//     fn propositions_to_indices() {
//         for pattern in 0u8..10u8 {
//             for digit in 0u8..10u8 {
//                 let prop = Proposition::PatternIsDigit { pattern, digit };
//                 assert_eq!(prop, Proposition::from_index(prop.to_index()));
//             }
//         }

//         for wire in 'a'..='g' {
//             for segment in 'a'..='g' {
//                 let prop = Proposition::WireIsSegment { wire, segment };
//                 assert_eq!(prop, Proposition::from_index(prop.to_index()));
//             }
//         }
//     }

//     #[test]
//     /// Test that convert from indices to propositions and back works.
//     fn indices_to_propositions() {
//         for index in 1..=Proposition::MAX_INDEX {
//             let prop = Proposition::from_index(index);
//             assert_eq!(index, prop.to_index());
//             assert_eq!(index.neg(), prop.negation_to_index());
//         }
//     }
// }
