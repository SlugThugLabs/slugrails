#![allow(warnings, clippy::all, clippy::pedantic)]

use slugrails::analyzer::{has_test_module, is_test_path, test_region_lines};

const TEST_MODULE_SOURCE: &str = "\
pub fn production() -> u8 {
    let x: Option<u8> = None;
    x.unwrap()
}

#[cfg(test)]
mod tests {
    #[test]
    fn t() {
        let y: Option<u8> = None;
        y.unwrap();
    }
}
";

#[test]
fn test_module_range_covers_the_whole_block() {
    assert_eq!(test_region_lines(TEST_MODULE_SOURCE), vec![(6, 13)]);
    assert!(has_test_module(TEST_MODULE_SOURCE));
}

#[test]
fn multiple_test_modules_are_reported() {
    let source = "\
#[cfg(test)]
mod a {
    fn one() {}
}

#[cfg(test)]
mod b {
    fn two() {}
}
";
    assert_eq!(test_region_lines(source), vec![(1, 4), (6, 9)]);
}

#[test]
fn cfg_test_on_a_statement_is_not_a_region() {
    let source = "#[cfg(test)]\nuse super::*;\n\nfn real() {}\n";
    assert!(test_region_lines(source).is_empty());
    assert!(!has_test_module(source));
}

#[test]
fn cfg_test_text_inside_a_string_is_not_a_region() {
    let source = "let s = \"#[cfg(test)] mod tests {\";\n";
    assert!(test_region_lines(source).is_empty());
}

#[test]
fn test_paths_follow_directory_and_suffix_conventions() {
    assert!(is_test_path("tests/foo.rs"));
    assert!(is_test_path("src/tests/foo.rs"));
    assert!(is_test_path("src/foo_test.rs"));
    assert!(is_test_path("test_helper.py"));
    assert!(is_test_path("components/Widget.spec.ts"));
}

#[test]
fn production_paths_containing_test_letters_are_not_tests() {
    assert!(!is_test_path("src/latest.rs"));
    assert!(!is_test_path("src/attestation.rs"));
    assert!(!is_test_path("src/contract/mod.rs"));
}
