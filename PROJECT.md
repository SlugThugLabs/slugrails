# Project: SlugRails Pure Rust Architectural Guardrails Engine

## Architecture
SlugRails is a pure Rust CLI tool and architectural guardrails engine designed to prevent architectural drift by turning agreed-upon software architecture into persistent, machine-enforceable constraints.

### Layer Topology (4-Tier DAG)
```
┌────────────────────────────────────────────────────────┐
│ Presentation Layer: cli, menu, server                  │
└──────────────────────────┬─────────────────────────────┘
                           │
┌──────────────────────────▼─────────────────────────────┐
│ Infrastructure Layer: storage, library                 │
└──────────────────────────┬─────────────────────────────┘
                           │
┌──────────────────────────▼─────────────────────────────┐
│ Core Domain Layer: analyzer, contract, guards, types   │
└──────────────────────────┬─────────────────────────────┘
                           │
┌──────────────────────────▼─────────────────────────────┐
│ Foundation Layer: error, style, util                   │
└────────────────────────────────────────────────────────┘
```

### Module Boundaries
1. `util`: Pure path normalization and string/character manipulation (no project dependencies).
2. `style`: ANSI terminal styling and color detection (foundation).
3. `error`: Strongly typed `SlugRailsError` enum with explicit propagation (foundation).
4. `types`: Core data structures (`ScanOutcome`, `Violation`, `Severity`, `Span`) (core).
5. `contract`: Architecture contract frontmatter parsing and BLAKE3 baseline protection (core).
6. `analyzer`: Pure Rust lexical tokenizer and structural metrics scanner (core).
7. `guards`: Guard runner and 13 specific architectural/structural guard checks (core).
8. `library`: Guard catalog discovery and builtin definitions (infrastructure).
9. `storage`: Baseline integrity and atomic JSON exception storage (infrastructure).
10. `cli`: Command-line interface subcommands and formatting (presentation).
11. `menu`: Interactive terminal setup workflow (presentation).
12. `server`: Model Context Protocol (MCP) JSON-RPC server (presentation).

## Feature Inventory
| # | Feature | Description | Milestone | Source |
|---|---------|-------------|-----------|--------|
| 1 | Standalone Pure Rust Binary | Standalone CLI binary `slugrails` with zero runtime daemon dependencies | M1 | Survey |
| 2 | Crate & Workspace Setup | `Cargo.toml` with edition 2024, forbid unsafe, and clippy denies | M1 | Survey |
| 3 | Foundation: Error Types | Strongly typed `SlugRailsError` and explicit Result propagation | M1 | Survey |
| 4 | Foundation: Paths & Utilities | Safe cross-platform path validation and normalization | M1 | Survey |
| 5 | Foundation: Terminal Styling | Zero-dependency ANSI formatting and NO_COLOR support | M1 | Survey |
| 6 | Foundation: Types & Outcomes | `ScanOutcome` (`Passed`, `Violations`, `Incomplete`, `Failed`) | M1 | Survey |
| 7 | Core: Pure Rust Tokenizer | Lexical scanner stripping comments, docstrings, raw strings without syn | M2 | Survey |
| 8 | Core: Function Metrics | Token-level function line counter (<= 50 lines) | M2 | Survey |
| 9 | Core: Nesting Metrics | Control-flow block nesting depth counter (<= 4 levels) | M2 | Survey |
| 10 | Core: Parameter Metrics | Function parameter counter (<= 5 parameters) | M2 | Survey |
| 11 | Core: File Walker | Bounded source file traversal with depth & gitignore awareness | M2 | Survey |
| 12 | Core: Contract Frontmatter | TOML frontmatter parser between `+++` fences in ARCHITECTURE.md | M2 | Survey |
| 13 | Core: Contract Validation | In-memory validation of declared modules and catalog guards | M2 | Survey |
| 14 | Guard: Source File Limits | Enforces <= 200 physical lines in production `src/**/*.rs` | M2 | Survey |
| 15 | Guard: Function Limits | Enforces <= 50 physical lines per function | M2 | Survey |
| 16 | Guard: Deep Nesting | Enforces control-flow nesting depth <= 4 | M2 | Survey |
| 17 | Guard: Parameter Count | Enforces function parameters <= 5 (excluding receiver) | M2 | Survey |
| 18 | Guard: Unsafe Code Ban | Enforces `#![forbid(unsafe_code)]` and bans `unsafe` blocks | M2 | Survey |
| 19 | Guard: No Panic / No Unwrap | Bans `panic!()`, `.unwrap()`, `.expect()` in production | M2 | Survey |
| 20 | Guard: No Stubs | Rejects `todo!()`, `unimplemented!()`, `dbg!()` in production | M2 | Survey |
| 21 | Guard: Swallowed Errors | Flags discarded results and `let _ =` error swallowing | M2 | Survey |
| 22 | Guard: Layer Dependencies | Enforces DAG layer import hierarchy per `allowed_dependencies` | M2 | Survey |
| 23 | Guard: Manifest Dependencies | Validates `Cargo.toml` dependencies against allowlist | M2 | Survey |
| 24 | Guard: Docs Drift | Verifies sync between `ARCHITECTURE.md` modules and `src/` files | M2 | Survey |
| 25 | Guard: No Secrets | Scans source for hardcoded API keys and tokens | M2 | Survey |
| 26 | Guard: No Duplicates | Disallows duplicate test names and duplicate commits | M2 | Survey |
| 27 | Infrastructure: Baseline Storage | BLAKE3 tamper-detection baseline at `.planning/slugrails/contract.baseline` | M3 | Survey |
| 28 | Infrastructure: Exception Storage | Atomic JSON exception storage at `.planning/slugrails/exceptions.json` | M3 | Survey |
| 29 | Infrastructure: Guard Catalog | Reusable guard catalog in `~/.slugthuglabs/slugrails/tests/` | M3 | Survey |
| 30 | CLI: Args Parser | Handwritten zero-dependency flag/option/subcommand parser | M3 | Survey |
| 31 | CLI: `slugrails check` | Verification engine with `--diff`, `--all`, `--fix`, `--format` | M3 | Survey |
| 32 | CLI: `slugrails init` | Scaffolds `.planning/ARCHITECTURE.md` and SlugPlan layout | M3 | Survey |
| 33 | CLI: `slugrails plan` | Interactive architecture discovery and contract generation | M3 | Survey |
| 34 | CLI: `slugrails scaffold-tests` | Generates `tests/slugrails.rs` integration test harness | M3 | Survey |
| 35 | CLI: `slugrails fix` | Auto-remediates debug prints in place | M3 | Survey |
| 36 | CLI: `slugrails hook` | Installs/uninstalls pre-commit gate in `.git/hooks/pre-commit` | M3 | Survey |
| 37 | CLI: `slugrails validate` | Validates contract against disk modules and catalog | M3 | Survey |
| 38 | CLI: `slugrails exception` | Manages authorized cryptographic exception tokens (requires TTY) | M3 | Survey |
| 39 | CLI: `slugrails menu` | Interactive setup and management TUI | M3 | Survey |
| 40 | CLI / Server: `slugrails serve` | Model Context Protocol (MCP) server over stdio/TCP | M3 | Survey |
| 41 | Diagnostics: Rustc-style Text | Formatted terminal output with line coordinates and remediation hints | M3 | Survey |
| 42 | Diagnostics: JSON Export | Machine-readable JSON diagnostic output | M3 | Survey |
| 43 | Diagnostics: SARIF Export | SARIF v2.1.0 output for GitHub Code Scanning | M3 | Survey |
| 44 | Acceptance: Line Coverage >= 85% | Source line test coverage >= 85% verified by `cargo-llvm-cov` | M4 | Survey |
| 45 | Acceptance: Static Pattern Check | 100% compliance with `.planning/scripts/check_rust.sh` on all `src/` files | M4 | Survey |
| 46 | Acceptance: Clean Build & Clippy | `cargo check`, `cargo clippy -- -D warnings`, `cargo fmt -- --check` clean | M4 | Survey |
| 47 | Verification: Negative Fixtures | Intentional violation test fixtures verifying checker failures | M5 | Survey |
| 48 | Verification: Full E2E Test Suite | 100% pass of Tiers 1-4 requirement-driven E2E tests | M5 | Survey |
| 49 | Verification: Adversarial Hardening | Tier 5 white-box edge case stress testing and adversarial hardening | M5 | Survey |

## Milestones
| # | Name | Scope | Dependencies | Status |
|---|------|-------|-------------|--------|
| M1 | Baseline Setup & Foundation | Port Cargo.toml, clippy.toml, foundation crates (`error`, `style`, `types`, `util`); refactor to pass `check_rust.sh` | None | PLANNED |
| M2 | Core Engine & Guard Checks | Port & refactor `analyzer`, `contract`, `guards`, `library`; flatten nesting <= 4, functions <= 50, params <= 5 | M1 | PLANNED |
| M3 | CLI Workflows, Storage & Server | Port & refactor `cli`, `storage`, `menu`, `server`; implement tri-format diagnostics and exception guards | M2 | PLANNED |
| M4 | Coverage Elevation & Static Audit | Add unit tests to raise line coverage from 65.10% to >= 85%; verify zero warnings on clippy/fmt; run `check_rust.sh` | M3 | PLANNED |
| M5 | E2E Test Suite Pass & Adversarial Hardening | Phase 1: Pass 100% of E2E tests (Tiers 1-4) published in `TEST_READY.md`. Phase 2: Adversarial hardening (Tier 5) | M4, E2E Track | PLANNED |

## Interface Contracts
### `analyzer` ↔ `guards`
- `Tokenizer`: `tokenize(content: &str) -> Result<Vec<Token>, LexError>`
- `Metrics`: `count_physical_lines(content: &str) -> usize`, `analyze_functions(content: &str) -> Vec<FunctionSpan>`, `calculate_nesting_depth(content: &str) -> usize`

### `contract` ↔ `cli`
- `parse_contract(content: &str) -> Result<ArchitectureContract, ContractError>`
- `verify_contract_baseline(root: &Path, content: &str) -> Result<(), ContractError>`

### `guards` ↔ `cli`
- `evaluate_guards(contract: &ArchitectureContract, files: &[PathBuf]) -> Result<ScanOutcome, GuardError>`

## Code Layout
```
C:\Users\Sly\slugrails/
├── Cargo.toml
├── clippy.toml
├── src/
│   ├── lib.rs
│   ├── main.rs
│   ├── analyzer/
│   ├── cli/
│   ├── contract/
│   ├── error.rs
│   ├── guards/
│   ├── library/
│   ├── menu/
│   ├── server/
│   ├── storage/
│   ├── style.rs
│   ├── types.rs
│   └── util/
├── tests/
│   ├── common/
│   └── *.rs
└── fixtures/
    ├── valid/
    └── violations/
```
