use slugrails::analyzer::{
    MarkerKind, find_action_markers, find_stub_markers, find_swallowed_errors,
};

#[test]
fn rust_stub_macros_are_detected() {
    let source = "fn f() {\n    todo!();\n    unimplemented!();\n    unreachable!();\n}\n";
    let markers = find_stub_markers(source);
    let kinds: Vec<MarkerKind> = markers.iter().map(|marker| marker.kind).collect();
    assert_eq!(
        kinds,
        vec![
            MarkerKind::Todo,
            MarkerKind::Unimplemented,
            MarkerKind::Unreachable
        ]
    );
    assert_eq!(markers[0].line, 2);
}

#[test]
fn panic_placeholder_is_a_stub_but_a_real_message_is_not() {
    let stub = find_stub_markers("fn f() {\n    panic!(\"not implemented\");\n}\n");
    assert_eq!(stub.len(), 1, "got {stub:?}");
    assert_eq!(stub[0].kind, MarkerKind::PanicNotImplemented);

    let real = find_stub_markers("fn f() {\n    panic!(\"disk full\");\n}\n");
    assert!(real.is_empty(), "got {real:?}");
}

#[test]
fn stubs_inside_comments_and_strings_are_ignored() {
    let source = "// todo!()\nlet s = \"unimplemented!()\";\n";
    assert!(find_stub_markers(source).is_empty());
}

#[test]
fn python_not_implemented_error_is_a_stub() {
    let markers = find_stub_markers("def f():\n    raise NotImplementedError\n");
    assert_eq!(markers.len(), 1, "got {markers:?}");
    assert_eq!(markers[0].kind, MarkerKind::NotImplementedError);
    assert_eq!(markers[0].line, 2);
}

#[test]
fn action_keywords_are_found_in_both_comment_styles() {
    let source = "// TODO: fix\n# FIXME: later\n/* HACK */\nlet x = 1; // ACTION\n";
    let markers = find_action_markers(source);
    let lines: Vec<usize> = markers.iter().map(|marker| marker.line).collect();
    assert_eq!(lines, vec![1, 2, 3, 4], "got {markers:?}");
    assert!(
        markers
            .iter()
            .all(|marker| marker.kind == MarkerKind::ActionItem)
    );
}

#[test]
fn action_keywords_in_strings_are_not_comments() {
    assert!(find_action_markers("let s = \"TODO\";\n").is_empty());
}

#[test]
fn rust_attributes_are_not_hash_comments() {
    let source = "#[derive(Debug)]\nstruct S;\n";
    assert!(find_action_markers(source).is_empty());
}

#[test]
fn discarded_results_and_empty_arms_are_detected() {
    let source = "\
fn f() -> Result<(), E> {
    let _ = do_thing();
    match r {
        _ => {}
    }
    if let Err(_) = r {}
    Ok(())
}
";
    let markers = find_swallowed_errors(source);
    let kinds: Vec<MarkerKind> = markers.iter().map(|marker| marker.kind).collect();
    assert!(
        kinds.contains(&MarkerKind::DiscardedResult),
        "got {markers:?}"
    );
    assert!(
        kinds.contains(&MarkerKind::EmptyMatchArm),
        "got {markers:?}"
    );
    assert!(kinds.contains(&MarkerKind::IgnoredError), "got {markers:?}");
}

#[test]
fn error_arms_and_empty_catches_are_detected() {
    let rust = "match r {\n    Err(_) => {}\n}\n";
    let markers = find_swallowed_errors(rust);
    assert_eq!(markers.len(), 1, "got {markers:?}");
    assert_eq!(markers[0].kind, MarkerKind::EmptyMatchArm);
    assert_eq!(markers[0].line, 2);

    let python = "try:\n    work()\nexcept:\n    pass\n";
    let markers = find_swallowed_errors(python);
    assert_eq!(markers.len(), 1, "got {markers:?}");
    assert_eq!(markers[0].kind, MarkerKind::EmptyCatch);

    let javascript = "try { work(); } catch (e) {}\n";
    let markers = find_swallowed_errors(javascript);
    assert_eq!(markers.len(), 1, "got {markers:?}");
    assert_eq!(markers[0].kind, MarkerKind::EmptyCatch);
}

#[test]
fn swallowed_shapes_in_strings_are_ignored() {
    assert!(find_swallowed_errors("let s = \"let _ = x\";\n").is_empty());
}
