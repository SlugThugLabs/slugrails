# SlugRails Phased Project Plan

**Current Status**: Phase 1 in progress (M1 dispatched)
**Last Updated**: 2026-10-10T02:48:30Z

---

## Phase 0: Survey & Discovery (Completed)
- [x] Mine architectural contract specifications from `.planning/ARCHITECTURE.md` (`spec_miner_arch_1`)
- [x] Mine CLI grammar, subcommands, and workflows from `.planning/VISION.md` (`spec_miner_vision_1`)
- [x] Audit workspace and reference repository `C:\Users\Sly\CodeGuards-mcp` against strict bounds (`explorer_codebase_1`)
- [x] Create master `PROJECT.md` and `.planning/projectplan.md`

---

## Phase 1: Baseline Setup & Foundation (Milestone M1)
- [ ] Initialize `Cargo.toml` (edition 2024, forbid unsafe, deny pedantic/unwrap/panic lints) and `clippy.toml`
- [ ] Port foundation modules: `src/lib.rs`, `src/error.rs`, `src/style.rs`, `src/types.rs`, `src/util/`
- [ ] Refactor foundation code:
  - [ ] Ensure all files in `src/` <= 200 physical lines
  - [ ] Ensure all functions <= 50 physical lines (split helper functions)
  - [ ] Flatten nesting depth <= 4 levels
  - [ ] Ensure function parameters <= 5 (refactor `scan_char_or_lifetime`, `handle_normal` etc.)
  - [ ] Ensure zero `unsafe`, zero `todo!()`, zero `unimplemented!()`, zero `mock`
  - [ ] Eliminate `\.clone\(\)` errors, `\.unwrap_or_default\(\)`, and single-letter variables
  - [ ] Add doc comments (`/// ...`) to all `pub` items
- [ ] Verify each foundation file against `.planning/scripts/check_rust.sh <file>`
- [ ] Phase gate: `cargo check` and compile actions pass cleanly

---

## Phase 2: Core Engine & Guard Checks (Milestone M2)
- [ ] Port core modules: `src/analyzer/`, `src/contract/`, `src/guards/`, `src/library/`
- [ ] Refactor long functions (> 50 lines) identified in survey:
  - [ ] `src/contract/scaffold.rs : plan_architecture` (108 lines -> helper functions)
  - [ ] `src/analyzer/metrics/functions/rust.rs : find_body_start` (101 lines)
  - [ ] `src/contract/discovery.rs : inspect_project` (74 lines)
  - [ ] `src/analyzer/metrics/lex/mod.rs : scan` (73 lines)
  - [ ] `src/analyzer/tokenizer.rs : tokenize_single_line` (62 lines)
  - [ ] `src/guards/checks/manifest_dependencies.rs : check_manifest_dependencies` (61 lines)
  - [ ] `src/library/builtins.rs : hygiene_guards` and `rust_guards` (59 lines)
  - [ ] `src/guards/runner.rs : evaluate_file_guards` (58 lines)
  - [ ] `src/guards/checks/layer_dependencies.rs : check_layer_dependencies` (53 lines)
  - [ ] `src/analyzer/walker.rs : collect_source_files` and `collect_git_diff_files` (87 & 53 lines)
- [ ] Flatten 38 nesting depth violations (> 4 levels) across analyzer and guard checks
- [ ] Run `.planning/scripts/check_rust.sh` on every core file before compiling
- [ ] Phase gate: `cargo check` passes cleanly

---

## Phase 3: Presentation, Storage & CLI Workflows (Milestone M3)
- [ ] Port presentation & infrastructure modules: `src/cli/`, `src/storage/`, `src/menu/`, `src/server/`, `src/main.rs`
- [ ] Refactor long functions:
  - [ ] `src/cli/exception.rs : run_exception` (88 lines)
  - [ ] `src/cli/init.rs : initialize_project` (83 lines)
  - [ ] `src/menu/steps.rs : connect_step` (76 lines)
  - [ ] `src/cli/sarif.rs : format_sarif` (65 lines)
  - [ ] `src/cli/format.rs : print_text_report` (64 lines)
  - [ ] `src/cli/hook.rs : install_hook` (61 lines)
  - [ ] `src/server/actions.rs : handle_create_guard_test` (60 lines)
  - [ ] `src/cli/fix.rs : run_fix` and `auto_remediate_violations` (54 & 51 lines)
- [ ] Verify zero `println!` outside `main.rs`
- [ ] Run `.planning/scripts/check_rust.sh` on all presentation files
- [ ] Phase gate: `cargo check` and `cargo clippy` pass with zero warnings

---

## Phase 4: Coverage Elevation & Acceptance Standards (Milestone M4)
- [ ] Split oversized test files (> 300 lines) from reference suite:
  - [ ] `guard_checks_suite_tests.rs` (815 lines)
  - [ ] `mcp_handlers_suite_tests.rs` (326 lines)
  - [ ] `mcp_stdio_tests.rs` (308 lines)
- [ ] Write targeted unit tests for zero/low coverage areas:
  - [ ] `src/menu/` and `src/main.rs` CLI entry points
  - [ ] `src/cli/validate.rs`, `src/cli/format.rs`, `src/style.rs`
  - [ ] `src/guards/checks/function_limits.rs`, `no_panic.rs`, `debug_prints.rs`, `stubs.rs`
- [ ] Measure line coverage with `cargo llvm-cov --text` to confirm >= 85.0%
- [ ] Run `cargo clippy --all-targets -- -D warnings` (zero warnings)
- [ ] Run `cargo fmt -- --check` (clean formatting)

---

## Phase 5: Autonomous Verification & E2E Validation (Milestone M5)
- [ ] Parallel E2E Testing Track complete: `TEST_READY.md` published
- [ ] Run E2E Test Suite (Tiers 1-4) against compiled binary `target/debug/slugrails.exe`:
  - [ ] Tier 1: Feature coverage across all 10 CLI subcommands
  - [ ] Tier 2: Boundary & corner cases (200/201 lines, 50/51 fn lines, 4/5 nesting)
  - [ ] Tier 3: Cross-feature workflows (`init` -> `plan` -> `check` -> `fix` -> `hook`)
  - [ ] Tier 4: Real-world application scenarios (pre-commit git hooks, corrupted states)
- [ ] Adversarial Negative Fixtures: verify intentional violations fail with actionable diagnostics and proper exit codes (1 for violation, 2 for incomplete)
- [ ] Tier 5 Adversarial Coverage Hardening (white-box stress testing)
- [ ] Final Forensic Audit verification (CLEAN)
- [ ] Handoff to Parent / Sentinel
