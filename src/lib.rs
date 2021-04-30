#![deny(clippy::all)]
#![deny(clippy::cargo)]
#![deny(clippy::pedantic)]
#![allow(clippy::default_trait_access)]

use argh::FromArgs;
use lazy_static::lazy_static;
use regex::Regex;
use std::str;

#[derive(FromArgs, Debug)]
/// `cargo-flake` -- a tool to detect flakey tests
pub struct Config {
    /// total global thread count
    #[argh(option)]
    pub threads: Option<usize>,

    /// space separated feature list
    #[argh(option)]
    pub features: Option<String>,

    /// only run tests with this prefix
    #[argh(option)]
    pub prefix: Option<String>,

    /// how many times to run individual tests
    #[argh(option)]
    pub iterations: Option<u16>,

    /// how many tests are allowed to fail
    #[argh(option)]
    pub tolerable_failures: Option<u16>,
}

lazy_static! {
    static ref TEST_NAME_RE: Regex =
        Regex::new("((?:[a-zA-Z0-9_]+[:]{2})*[a-zA-Z0-9_]+): test").unwrap();
}

#[must_use]
pub fn parse_test_names(input: &str) -> Vec<String> {
    let mut output = Vec::new();
    for cap in TEST_NAME_RE.captures_iter(input) {
        output.push(cap[1].into());
    }
    output
}

#[derive(Debug)]
pub struct TestSetup {
    pub name: String,
    pub command: String,
    pub iterations: u16,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub name: String,
    pub iterations: u16,
    pub successes: u16,
    pub failures: u16,
}

impl TestResult {
    #[must_use]
    pub fn new(name: String) -> Self {
        TestResult {
            name,
            iterations: 0,
            successes: 0,
            failures: 0,
        }
    }
}

#[cfg(test)]
mod test {
    use super::parse_test_names;

    #[test]
    fn test_name_match() {
        let text = "tests::a_test: test\n\nA_test: test\nnonsense text\na_test: test\ntls::settings::test::from_config_not_enabled: test\n: test";
        let names = parse_test_names(text);

        assert_eq!("tests::a_test", &names[0]);
        assert_eq!("A_test", &names[1]);
        assert_eq!("a_test", &names[2]);
        assert_eq!("tls::settings::test::from_config_not_enabled", &names[3]);
        assert_eq!(4, names.len());
    }

    #[test]
    fn test_long() {
        let mut sum: u128 = 1;
        for _ in 0..1_000_000_000 {
            sum = sum.saturating_add(sum);
            assert!(true)
        }
        assert!(sum > 2);
    }

    #[ignore]
    #[test]
    fn born_to_fail() {
        for _ in 0..1_000_000 {
            assert!(true)
        }
        assert!(false)
    }
}
