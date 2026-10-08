use codeguards_mcp::analyzer::nesting_depth;

#[test]
fn brace_nesting_is_per_function() {
    let source = "\
fn flat() {
    let x = 1;
}

fn nested() {
    if true {
        for _ in 0..1 {
        }
    }
}
";
    let sites = nesting_depth(source);
    let flat = sites.iter().find(|site| site.name == "flat").expect("flat");
    let nested = sites
        .iter()
        .find(|site| site.name == "nested")
        .expect("nested");
    assert_eq!(flat.max_depth, 1);
    assert_eq!(nested.max_depth, 3);
}

#[test]
fn braces_inside_strings_do_not_add_depth() {
    let source = "fn f() {\n    let s = \"{{{\";\n}\n";
    let sites = nesting_depth(source);
    assert_eq!(sites[0].max_depth, 1);
}

#[test]
fn python_nesting_counts_indentation() {
    let source = "def f():\n    if x:\n        return 1\n";
    let sites = nesting_depth(source);
    let f = sites.iter().find(|site| site.name == "f").expect("f");
    assert_eq!(f.max_depth, 2);
}
