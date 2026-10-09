# Changelog

All notable changes to SlugRails are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-10-08

### Added
- **10 Modular Builtin Guards**:
  - `complexity/source-limits`: Enforces configurable maximum code lines per file.
  - `complexity/function-limits`: Flags oversized functions and methods.
  - `complexity/deep-nesting`: Enforces maximum block nesting depth.
  - `languages/rust/no-unwrap`: Rejects bare `.unwrap()` and `.expect()` calls in non-test sources.
  - `languages/rust/unsafe-policy`: Rejects unexempted `unsafe` blocks and functions.
  - `hygiene/no-debug-prints`: Catches dangling `dbg!` and `console.log` statements.
  - `quality/no-stubs`: Detects placeholder macros (`todo!`, `unimplemented!`, stub panics).
  - `hygiene/swallowed-errors`: Flags swallowed errors (`let _ =` and empty matches).
  - `hygiene/no-secrets`: Detects hardcoded API keys, private keys, and cloud credentials.
  - `structural/layer-dependencies`: Enforces architectural layer dependencies declared in `.planning/ARCHITECTURE.md`.
- **Integrated SlugPlan Architecture Convergence Engine**:
  - `slugrails plan`: Conversational discovery of domain boundaries, automated layer generation, and test scaffolding.
  - MCP tool `slugplan_probe`: Enables autonomous planning agents to propose and converge contracts.
- **Standalone Test Scaffolder**:
  - `slugrails scaffold-tests`: Generates native `tests/slugrails.rs` for `cargo test` enforcement.
  - MCP tool `scaffold_test_harness`.
- **Adversarial Security Test Matrix**:
  - `tests/security_hostile_tests.rs`: Tests for Windows UNC, NTFS alternate streams, MS-DOS reserved names, and forged HMAC token rejection.
  - `tests/unicode_fuzz_tests.rs`: Tests for multi-byte UTF-8 tokenization (Cyrillic, Chinese, Emoji) and Python `#` comments.
- **Cross-Platform Distribution**:
  - `npm/package.json` and `npm/bin/slugrails.js` for zero-install execution via `npx slugrails`.
  - Enriched `Cargo.toml` package metadata, keywords, and categories.

### Security
- **Cryptographic HMAC Exception Verification**: Recomputes BLAKE3 keyed hash using central salt in `storage::is_exception_valid_with_ttl`. Forged or unkeyed tokens are rejected.
- **Autonomous Agent Guardrail Gate**: Added `human_confirmed: bool` check to `AddExceptionRequest` preventing autonomous agents from self-authorizing guardrail bypasses.
- **Filesystem Sandbox Hardening**: `validate_safe_path` actively rejects Windows UNC network paths (`\\`, `//`), NTFS alternate streams (`:`), and MS-DOS device names.
- **Subprocess Defense**: Added `-c core.fsmonitor=` and `-c core.hooksPath=/dev/null` to Git invocations in `collect_git_diff_files` to neutralize untrusted repository hooks.
- **Permissions Hardening**: Key generation in `get_or_init_secret_salt` enforces Unix `0o600` file permissions.

### Reliability & Performance
- **Zero Panic Paths**: Replaced raw byte slicing with Unicode character iteration (`chars().take(8)`) in `no_secrets.rs`, eliminating UTF-8 character boundary crashes.
- **Async Runtime Offloading**: Wrapped CPU-intensive Rayon guard scans in `tokio::task::spawn_blocking` in MCP handlers.
- **Directory Traversal Bounds**: Added `max_depth(32)` and a `50,000` file budget cap to `collect_source_files` to prevent file descriptor exhaustion.
- **Single Responsibility Lexer**: Extracted import syntax parsing to `src/analyzer/imports.rs`, reducing `tokenizer.rs` to 163 LOC.

### Changed
- **Repository Invariants**: All 61 files in `src/` strictly adhere to $\le 200$ LOC, `#![forbid(unsafe_code)]`, and zero test pollution in `src/`.
- **CLI Ergonomics**: Added hierarchical `--help` subcommands for `slugrails check` and aligned menu paths to `~/.slugthuglabs/bin`.
