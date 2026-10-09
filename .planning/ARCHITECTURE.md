+++
modules = ["analyzer", "contract", "error", "guards", "library", "server", "storage", "types", "util"]
layers = ["transport", "orchestration", "engine", "analysis", "storage", "foundation"]
enforce = [
  "source_limits",
  "function_limits",
  "deep_nesting",
  "no_stubs",
  "swallowed_errors",
  "no_secrets",
  "no_debug_prints",
  "no_unwrap",
  "unsafe_policy",
  "layer_dependencies",
]

[allowed_dependencies]
server = ["guards", "contract", "analyzer", "storage", "library", "types", "error", "util"]
guards = ["analyzer", "storage", "contract", "library", "types", "error", "util"]
contract = ["storage", "library", "types", "error", "util"]
analyzer = ["types", "error", "util"]
storage = ["types", "error", "util"]
library = ["types", "error", "util"]
types = []
error = []
util = ["types", "error"]

[[invariants]]
id = "INV-AUTH-01"
component = "storage"
contract = "Modifications to ARCHITECTURE.md without valid token hash MUST be rejected."
enforcement = "SHA-256 baseline verification"

[[invariants]]
id = "INV-EXEC-01"
component = "guards"
contract = "Enforced guards must run black-box without plaintext assertion leakage to prevent agent test tampering."
enforcement = "Compiled blackbox verification"

[[invariants]]
id = "INV-DRY-01"
component = "library"
contract = "New guards registered via create_guard_test must persist to ~/.slugthuglabs/slugrails/tests/ and be globally reusable."
enforcement = "Catalog indexing test"

[[invariants]]
id = "INV-SAFE-01"
component = "all"
contract = "Zero unsafe blocks across the entire codebase."
enforcement = "#![forbid(unsafe_code)]"

[[invariants]]
id = "INV-WIN-01"
component = "util"
contract = "Home directory and binary resolution must support Windows USERPROFILE and .exe extensions seamlessly."
enforcement = "Cross-platform path test"
+++

# SlugRails Architecture & Workflow Integration

> **Core Philosophy:** SlugRails is **NOT a linter replacement**—it is an **architectural bouncer and intent companion**.
>
> Linters (Clippy, ESLint, Ruff) enforce stylistic consistency, local syntax rules, and micro-optimizations within individual files.
>
> SlugRails operates at the architectural tier:
> - **Architectural Bouncer:** Validates structural module hierarchies, DAG dependency trees, cognitive nesting depth ceilings, unhandled/swallowed errors, credential leaks, and unsafe policies.
> - **Intent Companion:** Guides agents through human intent convergence using **SlugPlan** before code is written, locking down structural rails so autonomous agents do not drift.
> - Target repositories remain clean: guard definitions, baselines, and execution state live in the two-tier hub (`~/.slugthuglabs/slugrails/` and `.planning/slugrails/`).

---

## 1. Modular, DRY Guard-Test Taxonomy (`~/.slugthuglabs/slugrails/tests/`)

Tests are grouped into strict, orthogonal categories. Every test has a unique namespaced identifier: `<category>/<name>` (e.g. `complexity/source-limits`, `complexity/deep-nesting`, `hygiene/swallowed-errors`, `hygiene/no-secrets`, `languages/rust/unsafe-policy`).

```text
~/.slugthuglabs/slugrails/tests/
├── catalog.json                              <-- Auto-indexed search cache (tags, summaries)
│
├── structural/                               <-- System architecture, layers, & project truth
│   ├── docs-drift.guard.json                 <-- Bidirectional module-map vs. filesystem reality
│   ├── module-map.guard.json                 <-- Declared module path existence
│   ├── layer-dependencies.guard.json         <-- DAG import hierarchy (allowed_dependencies)
│   └── manifest-dependencies.guard.json      <-- Cargo.toml / package.json dependency whitelist
│
├── complexity/                               <-- Code volume, function size, & coupling
│   ├── source-limits.guard.json              <-- Production code line limits (default: 400 lines)
│   ├── function-limits.guard.json            <-- Max lines per function / method (default: 80 lines)
│   ├── deep-nesting.guard.json               <-- [NEWLY ACTIVE] Max control flow depth (default: 4)
│   └── fanout-coupling.guard.json            <-- Responsibility clustering & fan-out bounds
│
├── hygiene/                                  <-- General code cleanliness & safety
│   ├── no-duplicates.guard.json              <-- Duplicate test names & commit subjects
│   ├── swallowed-errors.guard.json           <-- [NEWLY ACTIVE] Discarded / unhandled error blocks
│   ├── no-secrets.guard.json                 <-- [NEWLY ACTIVE] API keys, tokens, and credentials
│   └── no-debug-prints.guard.json            <-- Leftover println!, console.log, dbg!
│
├── quality/                                  <-- Implementation integrity & documentation
│   ├── no-stubs.guard.json                   <-- Rejects TODO, FIXME, unimplemented!() in prod
│   ├── required-docstrings.guard.json        <-- Documentation coverage on public APIs
│   └── test-isolation.guard.json             <-- Tests isolated from production artifacts
│
├── languages/                                <-- Language-Specific Invariants (Pure Rust Tokenizers)
│   ├── rust/
│   │   ├── no-unwrap.guard.json              <-- Zero .unwrap() / .expect() in prod paths
│   │   ├── unsafe-policy.guard.json          <-- [NEWLY ACTIVE] Strict forbid(unsafe_code) enforcement
│   │   ├── tracing-instrument.guard.json     <-- #[tracing::instrument] on public entry points
│   │   ├── runtime-leak.guard.json           <-- Async (tokio/wasmtime/rayon) leaks into pure domain
│   │   ├── concurrency-ownership.guard.json  <-- Concurrency primitives bounds
│   │   ├── error-mapping.guard.json          <-- Typed error propagation across layer boundaries
│   │   ├── edition-lock.guard.json           <-- Toolchain / Rust edition integrity
│   │   └── public-api-bounds.guard.json      <-- Accidental pub fn / pub struct exports
│   └── python/
│       ├── type-annotations.guard.json       <-- Required type hints on function signatures
│       └── no-wildcard-imports.guard.json    <-- Disallows `from module import *`
│
└── custom/                                   <-- Project-Specific / Dynamic Custom Guard Tests
    └── <namespace>/
        └── <custom-test>.guard.json          <-- Interactive AI-authored guard tests
```

---

## 2. Newly Active Guard Tests

SlugRails integrates four newly active executable guard checks built directly into the pure-Rust tokenizer pipeline:

### 1. `complexity/deep-nesting`
* **Purpose:** Enforces an upper bound on nested control-flow constructs (loops, matches, if/else blocks; default ceiling: `<= 4`).
* **Why Linters Miss This:** Linters may count cyclomatic complexity heuristically without blocking builds. `deep-nesting` acts as a hard bouncer, terminating arrow anti-patterns and forcing agents to extract helper functions or use early guard returns.
* **Remediation:** "Flatten control flow using early returns, guard clauses, or extracted helper functions."

### 2. `hygiene/swallowed-errors`
* **Purpose:** Rejects discarded `Result` returns, empty `catch` blocks, and unhandled `Err(_) => {}` or `let _ = fallible_op()` patterns.
* **Why Linters Miss This:** Many linters permit blank match arms or wildcards. `swallowed-errors` requires explicit error propagation via `?`, structured conversion via `thiserror`, or logged diagnostics.
* **Remediation:** "Explicitly handle errors, propagate with '?', or log failures."

### 3. `hygiene/no-secrets`
* **Purpose:** Detects high-entropy strings, hardcoded bearer tokens, private keys (`BEGIN PRIVATE KEY`), AWS/GitHub/OpenAI credential prefixes, and embedded passwords.
* **Why Linters Miss This:** Traditional linters do not inspect string entropy or credential tokens. `no-secrets` prevents unintentional credential leaks before code hits staged commits.
* **Remediation:** "Extract hardcoded secret into environment variable or configuration file."

### 4. `languages/rust/unsafe-policy`
* **Purpose:** Enforces crate-level `#![forbid(unsafe_code)]` and flags unexempted `unsafe` blocks, unsafe functions, or unsafe traits across `src/`.
* **Why Linters Miss This:** Default toolchains allow `unsafe` code freely unless explicitly configured. `unsafe-policy` blocks memory safety escapes in production modules without an approved cryptographic exception.
* **Remediation:** "Replace unsafe block with safe abstraction or record an approved architecture exception."

---

## 3. Standalone Test Harness Generator (`slugrails scaffold-tests`)

To run invariant checking in local developer loops and CI pipelines without requiring active background daemons or running servers, SlugRails provides:

```bash
slugrails scaffold-tests
```

### Daemon-Free `cargo test` Execution
* **Harness Artifact:** Writes `tests/slugrails.rs` directly into the project repository.
* **No Daemon Required:** Standard `cargo test` or `cargo test --test slugrails` executes the compiled bouncer directly in-process. There is no need to run an MCP server or background daemon.
* **Tamper-Proof Invariants:** Because the test resides in `tests/slugrails.rs` while production code lives under `src/`, autonomous agents refactoring `src/` cannot weaken, comment out, or mock the test assertions.
* **Actionable Stderr Diagnostics:** When violations occur, the test harness emits formatted terminal diagnostics including the exact file location, offending token, guard rule ID, and concrete remediation advice before panicking.

---

## 4. Integrated SlugPlan Intent Convergence Engine & Architectural Interview

Autonomous coding is most fragile during initial design. If an agent begins writing code before architectural boundaries are agreed upon, structural drift is inevitable.

### The SlugPlan Engine
SlugPlan bridges human intent and machine enforcement through structured architectural interviews:

1. **CLI Architectural Interview (`slugrails plan`):**
   ```bash
   slugrails plan --intent "Order orchestration service with outbox event publisher" --style "hexagonal"
   ```
   * Accepts style hints: `layered` (`domain` -> `application` -> `infrastructure`), `hexagonal` (`domain` -> `ports` -> `adapters` -> `driver`), or `cli` (`domain` -> `engine` -> `cli`).
   * Synthesizes `.planning/ARCHITECTURE.md` with explicit `[allowed_dependencies]` matrices.
   * Records intent metadata into `.planning/slugplan/intent.json`.
   * Automatically scaffolds `tests/slugrails.rs` to lock in tests from turn one.

2. **Agent Workflow via MCP (`slugplan_probe` with `auto_scaffold`):**
   * AI agents invoke `slugplan_probe`:
     ```json
     {
       "description": "Multi-tenant authentication service",
       "language": "rust",
       "style": "layered",
       "auto_scaffold": true
     }
     ```
   * Evaluates language requirements, detects existing files, selects recommended guards (such as `unsafe-policy` and `no-secrets`), and when `auto_scaffold` is enabled, scaffolds both the contract and the test harness in a single conversational step.

---

## 5. Dynamic Custom Guard Test Creation (`create_guard_test`)

When a project has unique semantic or business invariants that standard guards don't cover:
1. **Agent identifies need during architecture discussion:** (e.g. *"All payment endpoints in `src/billing/` must require the `AuditLog` trait"*).
2. **Agent calls `create_guard_test`:**
   * Defines targeted token/pattern rules, path scopes, severity, and remediation advice.
   * Saves to `~/.slugthuglabs/slugrails/tests/custom/<namespace>/<test_name>.guard.json`.
3. **Persisted Globally:** The custom test is registered in `catalog.json` and immediately available across projects without bloating the target repository.

---

## 6. User-Authorized Exception Token Mechanism (`add_exception`)

### The Solution: Cryptographic / Hash-Verified User Tokens
1. **Agent Hits a Boundary:** If an exception is genuinely required, the check fails with exact instructions:
   ```text
   [SLUGRAILS-BLOCKED] complexity/source-limits
     File: src/server/transport.rs (450 code lines > max 400)
     Action Required: Ask user to run `slugrails exception add src/server/transport.rs complexity/source-limits --reason="..."`
   ```
2. **User Grants the Exception:**
   * Via CLI: `slugrails exception add <file> <guard> --reason="<why>"`
   * Or via User Slash Command / MCP tool: `/slugrails_add_exception` / `add_exception`
3. **Storage & Verification:**
   * Exceptions are stored securely in `.planning/slugrails/exceptions.json`.
   * Verified by matching token signature against file path and guard rule.
   * Stale tokens (where rule is no longer violated) are flagged for pruning via `slugrails exception prune`.

---

## 7. Pure-Rust Zero-Dependency Execution & Decoupled Triggers

* **Pure Rust Tokenizer:** Fast state-machine scanner (strips comments, docstrings, char/string literals) without heavy AST or Tree-sitter dependencies. **Measured 58.2 µs for 205 lines (284 lines/ms).**
* **Manifest Scanner:** Validates `Cargo.toml` dependency whitelists and forbids illegal crate coupling.
* **Decoupled Workflow:**
  * **Planning:** Agent + User use MCP authoring tools (`validate_architecture`, `slugplan_probe`, `create_guard_test`).
  * **Coding:** Agent writes normal code in `src/`. Zero SlugRails MCP overhead.
  * **Enforcement:** Git pre-commit hook runs `slugrails check` and `cargo test` executes `tests/slugrails.rs`.

---

## 8. Cross-Platform Parity (Windows & Unix)

SlugRails is built for first-class operation across Linux, macOS, and Windows:
* **Central Directory Resolution:** `get_slugthug_home()` resolves `SLUGTHUG_HOME` -> `HOME` -> `USERPROFILE` with zero hardcoded Unix root assumptions.
* **Binary Execution:** Platform-aware executable naming (`slugrails.exe` on Windows, `slugrails` on Unix) ensures install and connect CLI hooks work on all systems.
* **Path Canonicalization:** Safe path validation properly handles Windows drive letters, UNC paths, and extended-length prefixes (`\\?\`).

---

## 9. Repository Invariants Verification (`scripts/verify_invariants.ps1`)

The repository includes an automated verification script that rigorously validates internal invariants:
```powershell
pwsh -File .\scripts\verify_invariants.ps1
```
1. **File Line Budget:** Every file in `src/` has `<= 200 LOC` (hard ceiling).
2. **Zero Unsafe:** Zero occurrences of `unsafe` code in `src/` (enforced by crate-level `#![forbid(unsafe_code)]`).
3. **Zero Test Pollution:** Zero `#[cfg(test)]` or `#[test]` attributes in `src/` (tests isolated to `tests/`).
4. **Module Disk Alignment:** All modules declared in `src/guards/checks/mod.rs` exist on disk.
