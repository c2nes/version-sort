use std::cmp::Ordering;
use std::env;
use std::fs::File;
use std::fmt::{Display, Error, Formatter};
use std::io::{self, BufRead};

struct Version {
    s: String,
    parts: Vec<VersionPart>,
}

impl Version {
    fn parse(s: String) -> Version {
        let lower = s.to_lowercase();
        let mut parts = Vec::new();
        for part in lower.split(|c| c == '=' || c == '.' || c == '_') {
            let (qual, num) = Version::parse_part(part.to_string());
            parts.push(qual);
            parts.push(num);
        }
        Version { s, parts }
    }

    /// Parses a single part of a version string.
    ///
    /// Version parts are separated by dots, underscores or hyphens. A single
    /// version part can be numeric (e.g. "1"), a qualifier (e.g. "beta"), or
    /// a combination (e.g. "alpha3"). A part is considered a combination if
    /// it is composed of an entirely non-digit prefix with a numeric suffix.
    /// If digits and non-digits a freely mixed (e.g. "a1b2c3") then the part
    /// is considered a qualifier only, without a numeric suffix.
    ///
    /// This function parses the version part into a qualifier and numeric
    /// suffix pair, with a default version part being returned if the
    /// corresponding part (qualifier or numeric part) is not present.
    fn parse_part(part: String) -> (VersionPart, VersionPart) {
        let mut suffix_start = None;
        for (i, c) in part.char_indices() {
            if c.is_ascii_digit() {
                if suffix_start == None {
                    suffix_start = Some(i);
                }
            } else if suffix_start != None {
                suffix_start = None;
                break;
            }
        }

        match suffix_start {
            None => (VersionPart::new_qualifier(part), VersionPart::default()),
            Some(idx) => {
                let (q, n) = part.split_at(idx);
                let q = match q {
                    "" => VersionPart::default(),
                    _ => VersionPart::new_qualifier(q.to_owned())
                };
                let n = match n {
                    "" => VersionPart::default(),
                    _ => VersionPart::new_number(n.parse().unwrap())
                };

                (q, n)
            }
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct VersionPart {
    n: i64,
    q: String,
}

impl VersionPart {
    fn default() -> VersionPart {
        VersionPart {
            n: 0,
            q: "".to_owned(),
        }
    }

    fn new_number(n: i64) -> VersionPart {
        VersionPart { n, q: "".to_owned() }
    }

    fn new_qualifier(q: String) -> VersionPart {
        let n = match q.as_str() {
            "snapshot" => -5,
            "alpha" => -4,
            "beta" => -3,
            "rc" => -2,
            "cr" => -2,
            _ => -1,
        };
        VersionPart { n, q }
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let default = &VersionPart::default();
        for i in 0..self.parts.len().max(other.parts.len()) {
            let a = if i < self.parts.len() {
                &self.parts[i]
            } else {
                default
            };

            let b = if i < other.parts.len() {
                &other.parts[i]
            } else {
                default
            };

            let ord = a.cmp(&b);
            if ord != Ordering::Equal {
                return ord;
            }
        }

        self.parts.len().cmp(&other.parts.len())
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Version {}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.write_str(&self.s)
    }
}

fn main() {
    let stdin = io::stdin();
    let stdin = Box::new(stdin.lock()) as Box<dyn io::BufRead>;
    let input = match env::args_os().nth(1) {
        Some(path) => match File::open(&path) {
            Ok(file) => Box::new(io::BufReader::new(file)),
            Err(why) => panic!("could not open {:?}: {}", path, why)
        }
        None => stdin
    };

    let mut versions = input
        .lines()
        .map(Result::unwrap)
        .map(Version::parse)
        .collect::<Vec<_>>();
    versions.sort_unstable();
    for v in versions {
        println!("{}", v);
    }
}
