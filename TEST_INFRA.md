# SlugRails Test Infrastructure Specification

**Document:** `TEST_INFRA.md`  
**Applies To:** SlugRails Pure Rust Architectural Guardrails Engine  
**Status:** Authoritative Test Infrastructure Contract  
**Created:** 2026-10-10  

---

## 1. Overview & Dual Track Principles

SlugRails enforces architectural guardrails through a pure Rust engine. In accordance with the **Dual Track Engineering Pattern** specified in `PROJECT.md`, test development operates independently from and concurrently with production implementation:
- **Independent Test Design**: Tests are authored strictly against requirements in `ORIGINAL_REQUEST.md`, `.planning/VISION.md`, and `.planning/ARCHITECTURE.md`.
- **Opaque-Box Verification**: The test suite treats the `slugrails` binary and its public API as a black box, verifying externally observable outputs, formatted diagnostics, and exit codes:
  - `0 = PASS`: Clean architectural verification, all contracts satisfied.
  - `1 = VIOLATION`: Architectural rule breached (structural limits, safety, dependencies).
  - `2 = INCOMPLETE / ERROR`: Invalid command flags, missing contracts, unclosed frontmatter, corrupted baseline hashes, or missing catalog guards.
- **Strict Size Limits**: Every test file in `tests/` strictly adheres to the bound of **<= 300 physical lines** per file per `.planning/ARCHITECTURE.md` Section 3.
- **Fail-Closed Semantics**: An incomplete verification, truncated tree traversal, or missing guard catalog must NEVER report pass (exit code 0).

---

## 2. Directory Layout & Test Topology

```text
C:\Users\Sly\slugrails/
├── TEST_INFRA.md                    # This document
├── TEST_READY.md                    # Readiness publication marker
├── fixtures/
│   ├── valid/
│   │   └── clean_project/           # Compliant project fixture (clean DAG, valid limits)
│   └── violations/
│       ├── file_too_long/           # Source file > 200 physical lines (201 lines)
│       ├── function_too_long/       # Function > 50 physical lines (51 lines)
│       ├── forbidden_unsafe/        # Contains `unsafe` block / fn
│       ├── unwrap_used/             # Production code calling `.unwrap()`
│       ├── panic_used/              # Production code calling `panic!()`
│       ├── deep_nesting/            # Control-flow nesting depth > 4 (depth 5)
│       ├── parameter_overflow/      # Function parameters > 5 (6 parameters)
│       ├── illegal_layer_import/    # Layer boundary breach in 4-tier DAG
│       ├── swallowed_errors/        # Discarded Result via `let _ =`
│       ├── debug_prints/            # Production `dbg!()` macro calls
│       └── tampered_contract/       # Modified ARCHITECTURE.md with mismatched BLAKE3 baseline
└── tests/
    ├── common/
    │   └── mod.rs                   # Shared opaque-box execution harness (< 300 lines)
    ├── tier1_menu_test.rs           # Tier 1: `slugrails menu` feature tests
    ├── tier1_init_test.rs           # Tier 1: `slugrails init` feature tests
    ├── tier1_plan_test.rs           # Tier 1: `slugrails plan` feature tests
    ├── tier1_scaffold_test.rs       # Tier 1: `slugrails scaffold-tests` feature tests
    ├── tier1_check_test.rs          # Tier 1: `slugrails check` feature tests
    ├── tier1_fix_test.rs            # Tier 1: `slugrails fix` feature tests
    ├── tier1_hook_test.rs           # Tier 1: `slugrails hook` feature tests
    ├── tier1_validate_test.rs       # Tier 1: `slugrails validate` feature tests
    ├── tier1_exception_test.rs      # Tier 1: `slugrails exception` feature tests
    ├── tier1_serve_test.rs          # Tier 1: `slugrails serve` feature tests
    ├── tier2_structural_limits_test.rs # Tier 2: Exact 200/201 line & 50/51 fn line boundaries
    ├── tier2_nesting_params_test.rs    # Tier 2: Exact 4/5 nesting & 5/6 param boundaries
    ├── tier2_contract_baseline_test.rs # Tier 2: Missing/corrupted contracts & BLAKE3 baselines
    ├── tier2_empty_noninteractive_test.rs # Tier 2: Empty trees & non-interactive exception attempts
    ├── tier3_workflows_test.rs      # Tier 3: Pairwise cross-feature lifecycle combinations
    └── tier4_scenarios_test.rs      # Tier 4: Real-world scenarios (git diff, pre-commit, recovery)
```

---

## 3. Test Tiers & Acceptance Criteria

### Tier 1: Feature Coverage (>= 5 tests per CLI subcommand)
Verifies core functional behavior across all 10 CLI subcommands:
1. `menu`: Help output, terminal detection, exit codes, invalid sub-options, non-interactive abort.
2. `init`: Scaffolds `.planning/ARCHITECTURE.md`, preserves existing files, creates `intent.json`, discovers existing modules, rejects unwritable roots.
3. `plan`: Generates architecture draft, computes BLAKE3 baseline, validates discovered layers, handles custom project paths, rejects invalid arguments.
4. `scaffold-tests`: Creates `tests/slugrails.rs`, ensures idempotency, preserves existing test suites, verifies template contents, handles `--root`.
5. `check`: Enforces all rules (`--all`), respects `--diff` filtering, outputs text diagnostics, formats JSON (`--format json`), formats SARIF (`--format sarif`).
6. `fix`: Auto-remediates standalone `dbg!()` statements, preserves valid source, handles clean projects without alterations, respects `--diff`, validates return codes.
7. `hook`: Installs `.git/hooks/pre-commit`, enforces idempotency, preserves existing user hooks, cleanly uninstalls hook block, rejects non-git repositories.
8. `validate`: Validates contract against disk modules, reports missing catalog guards, reports undeclared modules, exits 0 on valid contract, exits 1 on schema violation.
9. `exception`: Authorizes tokens (`add`), requires interactive TTY, lists active tokens (`list`), revokes tokens (`revoke`), validates missing arguments.
10. `serve`: Boots MCP server over stdio, supports `--port`, returns protocol capabilities, shuts down cleanly, handles malformed JSON-RPC.

### Tier 2: Boundary & Corner Cases (>= 5 tests per boundary)
Verifies precise threshold boundaries and adversarial edge conditions:
1. **Source File Limits**: 199 lines (PASS), exact 200 lines (PASS), exact 201 lines (VIOLATION / exit code 1), empty lines boundary, comment lines boundary.
2. **Function Limits**: 49 lines (PASS), exact 50 lines (PASS), exact 51 lines (VIOLATION / exit code 1), multiline signature boundary, trailing comment boundary.
3. **Nesting Depth**: Depth 3 (PASS), exact depth 4 (PASS), exact depth 5 (VIOLATION / exit code 1), match expression depth, closure block depth.
4. **Parameter Count**: 4 parameters (PASS), exact 5 parameters (PASS), exact 6 parameters (VIOLATION / exit code 1), receiver `self` exclusion, generic parameter types.
5. **Empty Files & Trees**: Zero `.rs` files under `src/` (PASS with notification), empty `.rs` file (PASS), whitespace-only `.rs` file (PASS), missing `src/` folder (INCOMPLETE), empty `.planning/` directory (fallback).
6. **Contract Robustness**: Missing `ARCHITECTURE.md` (INCOMPLETE), missing `+++` fences (fallback empty contract), unclosed `+++` fences (ERROR / exit code 2), invalid TOML syntax (ERROR / exit code 2), undeclared module import (VIOLATION).
7. **Baseline Tampering**: Valid BLAKE3 baseline matches (PASS), 1-byte alteration in `ARCHITECTURE.md` (ERROR / exit code 2), corrupted baseline hash file (ERROR), missing baseline allowed on fresh init, re-locked baseline restores PASS.
8. **Exception Protection**: Non-interactive exception attempt (ERROR / exit code 2), missing exception reason (ERROR), invalid file argument (ERROR), expired exception token (VIOLATION), unrecognized token revocation (exit code 1).

### Tier 3: Cross-Feature Combinations
Verifies multi-command end-to-end user workflows:
1. `init` -> `plan` -> `check`: Scaffolds project, establishes baseline, verifies 0 violations on clean template.
2. `check` -> `fix` -> `check`: Detects `dbg!()` violation, executes auto-fix, confirms subsequent scan passes.
3. `hook install` -> `git commit` mock -> `hook uninstall`: Installs pre-commit gate, verifies script presence, removes gate, confirms original hook preserved.
4. `exception add` -> `check` (bypassed) -> `exception list` -> `exception revoke` -> `check` (violation): Lifecycle of temporary exception authorization.
5. `init` -> `validate` -> `scaffold-tests` -> `check`: Full project setup pipeline and validation.

### Tier 4: Real-World Scenarios
Verifies system behavior under complex production conditions:
1. **Dirty Git Worktrees**: Staged modifications scanned with `--diff`, unstaged files ignored, untracked files ignored.
2. **Pre-commit Gate Execution**: Simulated git commit execution blocking on architectural violations with clear terminal diagnostics.
3. **Full 4-Tier Project**: Multi-crate/multi-module codebase conforming to presentation -> infrastructure -> core -> foundation layer hierarchy.
4. **Corrupted Exception Storage**: Auto-recovery from malformed `exceptions.json` backing up to `exceptions.json.corrupt.<timestamp>` without process crash.
5. **Multiple Concurrent Violations**: File with simultaneous file length, function length, unsafe, and unwrap violations emits grouped, readable rustc-style diagnostics.

---

## 4. Test Execution Harness (`tests/common/mod.rs`)

The test harness provides:
- `run_slugrails(args: &[&str], cwd: &Path) -> CommandResult`: Executes the compiled `slugrails` binary or fallback runner, capturing exit code, stdout, and stderr.
- `create_test_workspace(name: &str) -> TestWorkspace`: Creates an isolated temporary directory with git support and cleanup on drop.
- `assert_exit_code(result: &CommandResult, expected: i32)`: Verifies exact exit codes.
- `assert_stdout_contains(result: &CommandResult, pattern: &str)`: Checks diagnostic output.
- `assert_stderr_contains(result: &CommandResult, pattern: &str)`: Checks error output.

---

## 5. Verification Commands

Run the full E2E test suite:
```powershell
cargo test --test tier1_*
cargo test --test tier2_*
cargo test --test tier3_*
cargo test --test tier4_*
cargo test --tests
```

Verify all test files satisfy physical line length bounds (<= 300 lines):
```powershell
Get-ChildItem -Path tests -Filter *.rs -Recurse | ForEach-Object {
    $lines = (Get-Content $_.FullName).Count
    if ($lines -gt 300) { Write-Error "$($_.Name) exceeds 300 lines: $lines" }
}
```
