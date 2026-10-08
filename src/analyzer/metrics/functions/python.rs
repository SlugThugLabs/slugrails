//! Python function span extraction and docstring detection.

use super::FunctionSpan;
use super::rust::{count_params, find_matching};
use crate::analyzer::metrics::lex::{CodeView, LazyRegex, re};
use regex::Regex;
use std::sync::LazyLock;

static RE_PY_DEF: LazyRegex = LazyLock::new(|| {
    Regex::new(r"(?:^|\n)[ \t]*(?:async[ \t]+)?def[ \t]+([A-Za-z_][A-Za-z0-9_]*)[ \t]*\(").ok()
});

pub fn python_function_spans(source: &str) -> Vec<FunctionSpan> {
    let mut spans = Vec::new();
    let Some(re_def) = re(&RE_PY_DEF) else {
        return spans;
    };
    let code = CodeView::build(source);
    let raw_lines: Vec<&str> = source.lines().collect();
    for captures in re_def.captures_iter(&code.text) {
        let Some(name_match) = captures.get(1) else {
            continue;
        };
        let Some(full_match) = captures.get(0) else {
            continue;
        };
        let name = name_match.as_str().to_string();
        let start_line = code.line_at(name_match.start());
        let open = full_match.end().saturating_sub(1);
        let Some(close) = find_matching(&code.text, open, b'(', b')') else {
            continue;
        };
        let param_count = count_params(code.text.get(open + 1..close).unwrap_or(""));
        let signature_end = code.line_at(close);
        let end_line = python_block_end(&raw_lines, start_line, signature_end);
        spans.push(FunctionSpan {
            name: name.clone(),
            start_line,
            end_line,
            param_count,
            is_pub: !name.starts_with('_'),
            has_doc: has_python_docstring(&raw_lines, signature_end),
        });
    }
    spans
}

fn python_block_end(raw_lines: &[&str], def_line: usize, signature_end: usize) -> usize {
    let Some(def_text) = raw_lines.get(def_line.saturating_sub(1)) else {
        return signature_end;
    };
    let def_indent = def_text.len() - def_text.trim_start().len();
    let mut end = signature_end;
    let mut index = signature_end;
    while let Some(text) = raw_lines.get(index) {
        let trimmed = text.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            index += 1;
            continue;
        }
        let indent = text.len() - text.trim_start().len();
        if indent <= def_indent {
            break;
        }
        end = index + 1;
        index += 1;
    }
    end
}

fn has_python_docstring(raw_lines: &[&str], signature_end: usize) -> bool {
    let mut index = signature_end;
    while let Some(text) = raw_lines.get(index) {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            index += 1;
            continue;
        }
        return trimmed.starts_with("\"\"\"")
            || trimmed.starts_with("'''")
            || trimmed.starts_with('"')
            || trimmed.starts_with('\'');
    }
    false
}
